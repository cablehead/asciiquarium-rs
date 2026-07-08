//! asciiquarium, in Rust. See README.md for the port notes.
//!
//! This is a vertical slice: a hand-rolled compositor, the entity model, and a
//! crossterm event loop that doubles as the frame clock -- enough to prove the
//! shape/mask/z-depth/transparency model with fish swimming and blowing
//! bubbles. The rest of the original's menagerie ports onto the same pieces.

mod art;
mod entity;
mod render;

use std::io::{self, Write};
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Color,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;

use entity::{Entity, Frame, Kind};

/// The water surface occupies these rows; fish live below it.
const WATER_TOP: i32 = 5;
const WATER_BOTTOM: i32 = WATER_TOP + art::WATER_SEGMENTS.len() as i32 - 1;

/// An aquarium animation in ASCII art.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Classic mode: use the original art set only.
    #[arg(short = 'c', long)]
    classic: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut term = TerminalGuard::enter()?;
    run(&mut term.out, cli.classic)
}

fn run(out: &mut impl Write, classic: bool) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let (mut w, mut h) = terminal::size()?;
    let mut screen = render::Screen::new(w, h);
    let mut entities = build_scene(w, h, classic, &mut rng);
    let mut paused = false;

    loop {
        // poll() is both the input reader and the ~10fps frame clock: it blocks
        // up to 100ms for an event, exactly like the original's halfdelay(1).
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(k) => match k.code {
                    // Raw mode swallows SIGINT, so handle Ctrl-C / Ctrl-D here.
                    KeyCode::Char('c' | 'd') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('r') => {
                        entities = build_scene(w, h, classic, &mut rng);
                    }
                    _ => {}
                },
                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                    screen = render::Screen::new(w, h);
                    entities = build_scene(w, h, classic, &mut rng);
                }
                _ => {}
            }
        }

        if !paused {
            tick(&mut entities, w, h, classic, &mut rng);
        }

        // Draw back-to-front: higher z first, lower z last (on top).
        screen.clear();
        entities.sort_by_key(|e| std::cmp::Reverse(e.z));
        for e in &entities {
            let f = e.current();
            screen.blit(
                e.x.round() as i32,
                e.y.round() as i32,
                &f.shape,
                &f.mask,
                e.default_color,
            );
        }
        screen.render(out)?;
    }
}

/// Advance the world one frame: move everything, emit and pop bubbles, and keep
/// the fish population topped up as they swim off-screen.
fn tick(entities: &mut Vec<Entity>, w: u16, h: u16, classic: bool, rng: &mut impl Rng) {
    for e in entities.iter_mut() {
        e.update(w, h);
    }

    // Fish occasionally blow a bubble (original: ~2% chance per tick).
    let mut spawns: Vec<Entity> = Vec::new();
    for e in entities.iter() {
        if e.kind == Kind::Fish && e.alive && rng.gen_range(0..100) > 97 {
            spawns.push(spawn_bubble(e));
        }
    }

    // A bubble pops when it reaches the surface.
    for e in entities.iter_mut() {
        if e.kind == Kind::Bubble && e.y.round() as i32 <= WATER_BOTTOM {
            e.alive = false;
        }
    }

    // Each fish that swam off-screen is replaced (the original's death_cb).
    let respawn = entities
        .iter()
        .filter(|e| e.kind == Kind::Fish && !e.alive)
        .count();
    entities.retain(|e| e.alive);
    for _ in 0..respawn {
        spawns.push(spawn_fish(w, h, classic, rng));
    }
    entities.extend(spawns);
}

/// Build the whole scene from scratch: the water surface plus a screen-sized
/// school of fish.
fn build_scene(w: u16, h: u16, classic: bool, rng: &mut impl Rng) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Tile each water segment across the full width.
    let seg_w = art::WATER_SEGMENTS[0].len();
    let repeat = w as usize / seg_w + 1;
    for (i, seg) in art::WATER_SEGMENTS.iter().enumerate() {
        entities.push(Entity {
            kind: Kind::Waterline,
            x: 0.0,
            y: (WATER_TOP + i as i32) as f64,
            z: 2 + i as i32,
            dx: 0.0,
            dy: 0.0,
            frames: vec![Frame::new(&seg.repeat(repeat), "")],
            frame: 0.0,
            frame_speed: 0.0,
            default_color: Color::DarkCyan,
            die_offscreen: false,
            alive: true,
        });
    }

    // Fish count scales with the water area, as in the original.
    let area = (h as i32 - 9).max(0) * w as i32;
    let fish_count = area / 350;
    for _ in 0..fish_count {
        entities.push(spawn_fish(w, h, classic, rng));
    }

    entities
}

/// Spawn one fish at a random depth, facing and speed, entering from the edge it
/// swims away from.
fn spawn_fish(w: u16, h: u16, _classic: bool, rng: &mut impl Rng) -> Entity {
    let right = rng.gen_bool(0.5);
    let (shape, mask) = if right {
        (art::FISH_R_SHAPE, art::FISH_R_MASK)
    } else {
        (art::FISH_L_SHAPE, art::FISH_L_MASK)
    };
    let mask = entity::resolve_fish_mask(mask, rng);
    let frame = Frame::new(shape, &mask);

    let speed = rng.gen_range(0.25..2.25);
    let dx = if right { speed } else { -speed };

    // Vertical band: below the surface, fully on-screen.
    let top = 9;
    let bottom = (h as i32 - frame.height()).max(top);
    let y = rng.gen_range(top..=bottom) as f64;

    // Enter from the side it is heading away from.
    let x = if right {
        1.0 - frame.width() as f64
    } else {
        w as f64 - 2.0
    };

    let z = rng.gen_range(3..20);

    Entity {
        kind: Kind::Fish,
        x,
        y,
        z,
        dx,
        dy: 0.0,
        frames: vec![frame],
        frame: 0.0,
        frame_speed: 0.0,
        default_color: Color::Yellow,
        die_offscreen: true,
        alive: true,
    }
}

/// Spawn a bubble at a fish's leading edge; it rises and grows as it goes.
fn spawn_bubble(fish: &Entity) -> Entity {
    let f = fish.current();
    let x = if fish.dx > 0.0 {
        fish.x + f.width() as f64
    } else {
        fish.x
    };
    let y = fish.y + (f.height() as f64 / 2.0);

    Entity {
        kind: Kind::Bubble,
        x,
        y,
        z: fish.z - 1, // always in front of its fish
        dx: 0.0,
        dy: -0.34,
        frames: art::BUBBLE.iter().map(|s| Frame::new(s, "")).collect(),
        frame: 0.0,
        frame_speed: 0.2,
        default_color: Color::Cyan,
        die_offscreen: true,
        alive: true,
    }
}

/// Puts the terminal into raw / alternate-screen mode and restores it on drop,
/// so a panic or early return never leaves the user's shell wrecked.
struct TerminalGuard {
    out: io::Stdout,
}

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, cursor::Hide)?;
        Ok(TerminalGuard { out })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.out, cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
