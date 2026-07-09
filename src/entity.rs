//! Entities: sprites with position, depth, velocity, lifecycle, and collision.
//!
//! This mirrors Term::Animation's entity model: multi-frame sprites with a
//! parallel color mask, fractional per-tick velocity, several ways to die, and
//! a death action that (as in the original's death_cb) spawns a successor.

use crossterm::style::Color;
use rand::Rng;

/// One animation frame: a shape and its line-aligned color mask.
#[derive(Clone)]
pub struct Frame {
    pub shape: Vec<String>,
    pub mask: Vec<String>,
}

impl Frame {
    /// Build a frame from a shape block and a mask block. A single leading
    /// newline (natural in the source art literals) is trimmed so row 0 of the
    /// shape lines up with row 0 of the mask.
    pub fn new(shape: &str, mask: &str) -> Self {
        Frame {
            shape: split_block(shape),
            mask: split_block(mask),
        }
    }

    pub fn width(&self) -> i32 {
        self.shape
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0) as i32
    }

    pub fn height(&self) -> i32 {
        self.shape.len() as i32
    }
}

/// Entity kind. Drives depth defaults and, for the physical ones, collisions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Waterline,
    Castle,
    Seaweed,
    Fish,
    Bubble,
    Teeth,
    Shark,
    Ship,
    Whale,
    Monster,
    BigFish,
    Splat,
}

/// What to spawn when this entity dies, the port of Term::Animation's
/// `death_cb`. The tick loop interprets these after removing the corpse.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OnDeath {
    Nothing,
    Fish,
    Seaweed,
    RandomObject,
}

pub struct Entity {
    pub etype: EntityType,
    pub x: f64,
    pub y: f64,
    pub z: i32,
    pub dx: f64,
    pub dy: f64,
    pub frames: Vec<Frame>,
    pub frame: f64,
    pub frame_speed: f64,
    pub default_color: Color,
    /// Spaces are transparent when true (Term::Animation's `auto_trans`).
    pub auto_trans: bool,
    /// The explicit transparency character (Term::Animation default: '?').
    pub trans: char,
    /// Participates in collision detection.
    pub physical: bool,
    pub die_offscreen: bool,
    /// Die after this many ticks alive (Term::Animation `die_frame`).
    pub die_frame: Option<u32>,
    /// Die at this absolute global tick (our stand-in for `die_time`).
    pub die_tick: Option<u64>,
    pub on_death: OnDeath,
    pub age: u32,
    pub alive: bool,
}

impl Entity {
    pub fn current(&self) -> &Frame {
        let n = self.frames.len();
        &self.frames[(self.frame as usize) % n]
    }

    /// True if this shape character does not draw (lets lower-z cells show).
    pub fn is_transparent(&self, ch: char) -> bool {
        ch == self.trans || (self.auto_trans && ch == ' ')
    }

    /// Absolute screen cells this entity currently occupies (non-transparent).
    pub fn cells(&self) -> Vec<(i32, i32)> {
        let f = self.current();
        let mut out = Vec::new();
        for (row, line) in f.shape.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if !self.is_transparent(ch) && ch != ' ' {
                    out.push((
                        self.x.round() as i32 + col as i32,
                        self.y.round() as i32 + row as i32,
                    ));
                }
            }
        }
        out
    }

    /// Advance one tick: move, cycle frames, age, and apply the death rules.
    pub fn update(&mut self, tick: u64, screen_w: u16, screen_h: u16) {
        self.x += self.dx;
        self.y += self.dy;
        self.frame += self.frame_speed;
        self.age += 1;

        if self.die_offscreen {
            let f = self.current();
            let off_x = self.x + f.width() as f64 <= 0.0 || self.x >= screen_w as f64;
            let off_y = self.y + f.height() as f64 <= 0.0 || self.y >= screen_h as f64;
            if off_x || off_y {
                self.alive = false;
            }
        }
        if let Some(df) = self.die_frame {
            if self.age >= df {
                self.alive = false;
            }
        }
        if let Some(dt) = self.die_tick {
            if tick >= dt {
                self.alive = false;
            }
        }
    }
}

/// Split a text block into lines, trimming a single leading newline.
fn split_block(s: &str) -> Vec<String> {
    let s = s.strip_prefix('\n').unwrap_or(s);
    s.lines().map(|l| l.to_string()).collect()
}

/// The 12-color palette digits are randomized into.
const PALETTE: [char; 12] = ['c', 'C', 'r', 'R', 'y', 'Y', 'b', 'B', 'g', 'G', 'm', 'M'];

/// Randomize a color mask: assign each digit 1..9 a single random palette color
/// (so every cell sharing a digit shares a color), leaving letters and spaces.
///
/// This reproduces the Perl `rand_color`, including its off-by-one: it indexes
/// with `rand($#colors)`, i.e. 0..=10, so the 12th color ('M') is never chosen.
/// We keep the bug for fidelity.
pub fn rand_color(mask: &str, rng: &mut impl Rng) -> String {
    resolve(mask, rng, &[])
}

/// Like `rand_color`, but first forces the eye (digit 4) to white, as the fish
/// spawner does before randomizing.
pub fn resolve_fish_mask(mask: &str, rng: &mut impl Rng) -> String {
    resolve(mask, rng, &[('4', 'W')])
}

fn resolve(mask: &str, rng: &mut impl Rng, forced: &[(char, char)]) -> String {
    let mut map = std::collections::HashMap::new();
    for &(k, v) in forced {
        map.insert(k, v);
    }
    mask.chars()
        .map(|c| {
            if c.is_ascii_digit() && c != '0' {
                // 0..PALETTE.len()-1 preserves the original's off-by-one bug.
                *map.entry(c)
                    .or_insert_with(|| PALETTE[rng.gen_range(0..PALETTE.len() - 1)])
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_newline_trimmed_and_dims_measured() {
        let f = Frame::new("\nAB\nC\n", "");
        assert_eq!(f.shape, vec!["AB".to_string(), "C".to_string()]);
        assert_eq!(f.width(), 2);
        assert_eq!(f.height(), 2);
    }

    #[test]
    fn mask_is_stable_per_digit_eye_white() {
        let mut rng = rand::thread_rng();
        let out = resolve_fish_mask("11 4 22", &mut rng);
        let chars: Vec<char> = out.chars().collect();
        assert_eq!(chars[0], chars[1]); // both "1"s resolve to the same color
        assert_eq!(chars[3], 'W'); // eye
        assert_ne!(chars[5], '2'); // digit was replaced
        assert_eq!(chars[5], chars[6]);
    }

    #[test]
    fn question_mark_is_transparent_when_set() {
        let e = Entity {
            etype: EntityType::Fish,
            x: 0.0,
            y: 0.0,
            z: 0,
            dx: 0.0,
            dy: 0.0,
            frames: vec![Frame::new("a?b", "")],
            frame: 0.0,
            frame_speed: 0.0,
            default_color: Color::Reset,
            auto_trans: true,
            trans: '?',
            physical: false,
            die_offscreen: false,
            die_frame: None,
            die_tick: None,
            on_death: OnDeath::Nothing,
            age: 0,
            alive: true,
        };
        assert!(e.is_transparent('?'));
        assert!(e.is_transparent(' '));
        assert!(!e.is_transparent('a'));
        // Occupies (0,0) and (2,0) but not the transparent (1,0).
        assert_eq!(e.cells(), vec![(0, 0), (2, 0)]);
    }
}
