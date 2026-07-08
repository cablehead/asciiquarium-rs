//! Spawners: the port of the original's `add_*` subroutines.
//!
//! Each function builds one creature (sometimes several entities -- the shark
//! and its collision "teeth") from the byte-exact art in `art`. Positions,
//! speeds, depths, and the new/classic art selection follow the Perl.

use crossterm::style::Color;
use rand::Rng;

use crate::art;
use crate::entity::{rand_color, resolve_fish_mask, Entity, EntityType, Frame, OnDeath};

// Depths, lower draws on top (see the original's %depth table).
const Z_SHARK: i32 = 2;
const Z_TEETH: i32 = 3;
const Z_SEAWEED: i32 = 21;
const Z_CASTLE: i32 = 22;
const Z_WATER_GAP1: i32 = 7; // ship
const Z_WATER_GAP2: i32 = 5; // whale, sea monster
const Z_BIG_FISH: i32 = 2;
const WATER_TOP: i32 = 5;

/// Roughly ten ticks per second, used to convert the seaweed's minutes-long
/// lifetime into ticks.
const TICKS_PER_SEC: u64 = 10;

/// An `Entity` with neutral defaults; spawners override what they need.
fn ent(etype: EntityType, x: f64, y: f64, z: i32, frames: Vec<Frame>) -> Entity {
    Entity {
        etype,
        x,
        y,
        z,
        dx: 0.0,
        dy: 0.0,
        frames,
        frame: 0.0,
        frame_speed: 0.0,
        default_color: Color::Reset,
        auto_trans: false,
        trans: '?',
        physical: false,
        die_offscreen: false,
        die_frame: None,
        die_tick: None,
        on_death: OnDeath::Nothing,
        age: 0,
        alive: true,
    }
}

/// `rng.gen_range(lo..hi)`, but safe when the range is empty on tiny terminals.
fn between(rng: &mut impl Rng, lo: i32, hi: i32) -> i32 {
    if hi <= lo {
        lo
    } else {
        rng.gen_range(lo..hi)
    }
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

/// The water surface: four tiled segment rows at descending depth.
pub fn water(w: u16) -> Vec<Entity> {
    let seg_w = art::WATER[0].len();
    let repeat = w as usize / seg_w + 1;
    let z = [8, 6, 4, 2]; // water_line0..3
    art::WATER
        .iter()
        .enumerate()
        .map(|(i, seg)| {
            let mut e = ent(
                EntityType::Waterline,
                0.0,
                (WATER_TOP + i as i32) as f64,
                z[i],
                vec![Frame::new(&seg.repeat(repeat), "")],
            );
            e.default_color = Color::DarkCyan;
            e.physical = true;
            e
        })
        .collect()
}

/// The castle, anchored to the bottom-right corner.
pub fn castle(w: u16, h: u16) -> Entity {
    let mut e = ent(
        EntityType::Castle,
        (w as i32 - 32) as f64,
        (h as i32 - 13) as f64,
        Z_CASTLE,
        vec![Frame::new(art::CASTLE[0], art::CASTLE[1])],
    );
    // 'BLACK' in the original is bold-black, i.e. grey; filler spaces are
    // opaque but a space's color never shows, so the body renders in grey.
    e.default_color = Color::DarkGrey;
    e
}

/// One strand of seaweed: two alternating frames, a random height, a random
/// spot along the floor, and a several-minute lifetime that respawns a strand.
pub fn seaweed(w: u16, h: u16, tick: u64, rng: &mut impl Rng) -> Entity {
    let height = between(rng, 3, 7); // int(rand(4)) + 3
    let (mut f0, mut f1) = (String::new(), String::new());
    for i in 1..=height {
        // Odd rows lean one way, even rows the other.
        if i % 2 == 1 {
            f1.push_str("(\n");
            f0.push_str(" )\n");
        } else {
            f0.push_str("(\n");
            f1.push_str(" )\n");
        }
    }
    let x = between(rng, 1, w as i32 - 1) as f64;
    let y = (h as i32 - height) as f64;
    let speed = rng.gen_range(0.25..0.30); // rand(.05) + .25

    let mut e = ent(
        EntityType::Seaweed,
        x,
        y,
        Z_SEAWEED,
        vec![Frame::new(&f0, ""), Frame::new(&f1, "")],
    );
    e.default_color = Color::DarkGreen;
    e.frame_speed = speed;
    // Lives 8 to 12 minutes, then a fresh strand replaces it.
    let secs = between(rng, 8 * 60, 12 * 60) as u64;
    e.die_tick = Some(tick + secs * TICKS_PER_SEC);
    e.on_death = OnDeath::Seaweed;
    e
}

// ---------------------------------------------------------------------------
// Fish and bubbles
// ---------------------------------------------------------------------------

/// One fish, at a random depth, facing, speed, and (in non-classic mode) art
/// set. Even art indices face right; odd face left.
pub fn fish(w: u16, h: u16, classic: bool, rng: &mut impl Rng) -> Entity {
    let new = !classic && rng.gen_range(0..12) > 8; // ~25%
    let arr: &[&str] = if new { &art::NEW_FISH } else { &art::OLD_FISH };
    let count = arr.len() / 2;
    let fish_num = rng.gen_range(0..count);
    let idx = fish_num * 2;
    let right = fish_num % 2 == 0;

    let mask = resolve_fish_mask(arr[idx + 1], rng);
    let frame = Frame::new(arr[idx], &mask);
    let (fw, fh) = (frame.width(), frame.height());

    let mut speed = rng.gen_range(0.25..2.25);
    if !right {
        speed = -speed;
    }
    let z = between(rng, 3, 20);
    let y = between(rng, 9, h as i32 - fh) as f64;
    let x = if right {
        (1 - fw) as f64
    } else {
        (w as i32 - 2) as f64
    };

    let mut e = ent(EntityType::Fish, x, y, z, vec![frame]);
    e.dx = speed;
    e.default_color = Color::White;
    e.auto_trans = true;
    e.physical = true;
    e.die_offscreen = true;
    e.on_death = OnDeath::Fish;
    e
}

/// A bubble at a fish's leading edge; it rises and grows, and pops at the
/// surface (handled by the collision pass in main).
pub fn bubble(fish: &Entity) -> Entity {
    let f = fish.current();
    let x = if fish.dx > 0.0 {
        fish.x + f.width() as f64
    } else {
        fish.x
    };
    let y = fish.y + (f.height() as f64 / 2.0);

    let frames = art::BUBBLE.iter().map(|s| Frame::new(s, "")).collect();
    let mut e = ent(EntityType::Bubble, x, y, fish.z - 1, frames);
    e.dy = -1.0;
    e.frame_speed = 0.1;
    e.default_color = Color::Cyan;
    e.trans = ' ';
    e.physical = true;
    e.die_offscreen = true;
    e
}

/// A short-lived splat where a shark's teeth met a fish.
pub fn splat(x: i32, y: i32, z: i32) -> Entity {
    let frames = art::SPLAT.iter().map(|s| Frame::new(s, "")).collect();
    let mut e = ent(
        EntityType::Splat,
        (x - 4) as f64,
        (y - 2) as f64,
        z - 2,
        frames,
    );
    e.default_color = Color::Red;
    e.trans = ' ';
    e.frame_speed = 0.25;
    e.die_frame = Some(15);
    e
}

// ---------------------------------------------------------------------------
// The roaming specials
// ---------------------------------------------------------------------------

/// Pick and build one random "special". Its death spawns the next, so exactly
/// one roams at a time.
pub fn random_object(w: u16, h: u16, classic: bool, rng: &mut impl Rng) -> Vec<Entity> {
    match rng.gen_range(0..5) {
        0 => vec![ship(w, h, rng)],
        1 => whale(w, h, rng),
        2 => monster(w, h, classic, rng),
        3 => vec![big_fish(w, h, classic, rng)],
        _ => shark(w, h, rng),
    }
}

/// A shark plus its invisible-ish "teeth": a single physical `*` at the mouth
/// that does the eating.
pub fn shark(w: u16, h: u16, rng: &mut impl Rng) -> Vec<Entity> {
    let dir = rng.gen_range(0..2);
    let (mut x, mut speed) = (-53.0_f64, 2.0_f64);
    let y = between(rng, 9, h as i32 - 10);
    let mut teeth_x = -9;
    if dir == 1 {
        speed = -2.0;
        x = (w as i32 - 2) as f64;
        teeth_x = x as i32 + 9;
    }
    let teeth_y = y + 7;

    let mut teeth = ent(
        EntityType::Teeth,
        teeth_x as f64,
        teeth_y as f64,
        Z_TEETH,
        vec![Frame::new("*", "")],
    );
    teeth.dx = speed;
    teeth.default_color = Color::White;
    teeth.physical = true;

    let mut s = ent(
        EntityType::Shark,
        x,
        y as f64,
        Z_SHARK,
        vec![Frame::new(art::SHARK[dir], art::SHARK[2 + dir])],
    );
    s.dx = speed;
    s.default_color = Color::Cyan;
    s.auto_trans = true;
    s.die_offscreen = true;
    s.on_death = OnDeath::RandomObject;

    vec![teeth, s]
}

/// A ship sailing along the waterline.
pub fn ship(w: u16, h: u16, rng: &mut impl Rng) -> Entity {
    let _ = h;
    let dir = rng.gen_range(0..2);
    let (mut x, mut speed) = (-24.0_f64, 1.0_f64);
    if dir == 1 {
        speed = -1.0;
        x = (w as i32 - 2) as f64;
    }
    let mut e = ent(
        EntityType::Ship,
        x,
        0.0,
        Z_WATER_GAP1,
        vec![Frame::new(art::SHIP[dir], art::SHIP[2 + dir])],
    );
    e.dx = speed;
    e.default_color = Color::White;
    e.auto_trans = true;
    e.die_offscreen = true;
    e.on_death = OnDeath::RandomObject;
    e
}

/// A whale that surfaces and blows an animated spout.
pub fn whale(w: u16, h: u16, rng: &mut impl Rng) -> Vec<Entity> {
    let _ = h;
    let dir = rng.gen_range(0..2);
    let (x, speed, align) = if dir == 1 {
        ((w as i32 - 2) as f64, -1.0, 1)
    } else {
        (-18.0, 1.0, 11)
    };

    let whale_img = art::WHALE[dir];
    let whale_mask = art::WHALE[2 + dir];
    let spouts = &art::WHALE[4..11]; // 7 spout frames

    let mut frames = Vec::with_capacity(12);
    // Five frames with no spout: the whale sits three rows lower to line up
    // with the spout frames.
    for _ in 0..5 {
        frames.push(Frame::new(&format!("\n\n\n{whale_img}"), whale_mask));
    }
    // Seven frames animating the spout above the whale.
    for spout in spouts {
        let sep = format!("\n{}", " ".repeat(align));
        let aligned = perl_split_nl(spout).join(&sep);
        frames.push(Frame::new(&format!("{aligned}{whale_img}"), whale_mask));
    }

    let mut e = ent(EntityType::Whale, x, 0.0, Z_WATER_GAP2, frames);
    e.dx = speed;
    e.frame_speed = 1.0;
    e.default_color = Color::White;
    e.auto_trans = true;
    e.die_offscreen = true;
    e.on_death = OnDeath::RandomObject;
    vec![e]
}

/// A sea monster, undulating across the surface. New or classic art set.
pub fn monster(w: u16, h: u16, classic: bool, rng: &mut impl Rng) -> Vec<Entity> {
    let _ = h;
    let dir = rng.gen_range(0..2);
    let speed = if dir == 1 { -2.0 } else { 2.0 };

    let frames: Vec<Frame> = if classic {
        // OLD_MONSTER: [d0 x4, d1 x4, mask_d0, mask_d1]
        let base = dir * 4;
        let mask = art::OLD_MONSTER[8 + dir];
        (0..4)
            .map(|f| Frame::new(art::OLD_MONSTER[base + f], mask))
            .collect()
    } else {
        // NEW_MONSTER: [d0 x2, d1 x2, mask_d0, mask_d1]
        let base = dir * 2;
        let mask = art::NEW_MONSTER[4 + dir];
        (0..2)
            .map(|f| Frame::new(art::NEW_MONSTER[base + f], mask))
            .collect()
    };

    let x = if dir == 1 {
        (w as i32 - 2) as f64
    } else if classic {
        -64.0
    } else {
        -54.0
    };

    let mut e = ent(EntityType::Monster, x, 2.0, Z_WATER_GAP2, frames);
    e.dx = speed;
    e.frame_speed = 0.25;
    e.default_color = Color::Green;
    e.auto_trans = true;
    e.die_offscreen = true;
    e.on_death = OnDeath::RandomObject;
    vec![e]
}

/// A big fish. In non-classic mode there is a 1-in-3 chance of the second one.
pub fn big_fish(w: u16, h: u16, classic: bool, rng: &mut impl Rng) -> Entity {
    let second = !classic && rng.gen_range(0..3) > 1; // ~1/3
    let (art_pair, base_speed, left_x, min_gap): (&[&str], f64, f64, i32) = if second {
        (&art::BIG_FISH_2, 2.5, -33.0, 14)
    } else {
        (&art::BIG_FISH_1, 3.0, -34.0, 15)
    };

    let dir = rng.gen_range(0..2);
    let (x, speed) = if dir == 1 {
        ((w as i32 - 1) as f64, -base_speed)
    } else {
        (left_x, base_speed)
    };
    let y = between(rng, 9, h as i32 - min_gap) as f64;
    let mask = rand_color(art_pair[2 + dir], rng);

    let mut e = ent(
        EntityType::BigFish,
        x,
        y,
        Z_BIG_FISH,
        vec![Frame::new(art_pair[dir], &mask)],
    );
    e.dx = speed;
    e.default_color = Color::Yellow;
    e.auto_trans = true;
    e.die_offscreen = true;
    e.on_death = OnDeath::RandomObject;
    e
}

/// Perl's `split("\n", s)`: split on newlines and drop trailing empty fields.
fn perl_split_nl(s: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = s.split('\n').collect();
    while parts.last() == Some(&"") {
        parts.pop();
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perl_split_drops_trailing_empties() {
        assert_eq!(perl_split_nl("\n\n   :\n"), vec!["", "", "   :"]);
        assert_eq!(perl_split_nl("a\nb"), vec!["a", "b"]);
    }

    #[test]
    fn whale_has_twelve_frames() {
        let mut rng = rand::thread_rng();
        let v = whale(200, 40, &mut rng);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].frames.len(), 12);
    }

    #[test]
    fn shark_spawns_teeth_and_body() {
        let mut rng = rand::thread_rng();
        let v = shark(200, 40, &mut rng);
        assert_eq!(v.len(), 2);
        assert!(v.iter().any(|e| e.etype == EntityType::Teeth));
        assert!(v.iter().any(|e| e.etype == EntityType::Shark));
    }
}
