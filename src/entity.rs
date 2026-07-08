//! Entities: sprites with position, depth, velocity, and multi-frame shapes.
//!
//! This mirrors Term::Animation's entity model closely enough that porting the
//! rest of the `add_*` spawners from the Perl is mechanical. Each entity owns
//! its frames (shape + parallel color mask), moves by a fractional per-tick
//! velocity, and can die off-screen.

use crossterm::style::Color;
use rand::Rng;

/// One animation frame: a shape and its line-aligned color mask.
#[derive(Clone)]
pub struct Frame {
    pub shape: Vec<String>,
    pub mask: Vec<String>,
}

impl Frame {
    /// Build a frame from a shape block and an optional mask block. A leading
    /// newline (natural when writing `"\n...."` literals) is trimmed so row 0
    /// lines up with the mask.
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

/// What an entity is, which drives its per-tick behavior.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Waterline,
    Fish,
    Bubble,
}

pub struct Entity {
    pub kind: Kind,
    pub x: f64,
    pub y: f64,
    pub z: i32,
    pub dx: f64,
    pub dy: f64,
    pub frames: Vec<Frame>,
    pub frame: f64,
    pub frame_speed: f64,
    pub default_color: Color,
    pub die_offscreen: bool,
    pub alive: bool,
}

impl Entity {
    pub fn current(&self) -> &Frame {
        let n = self.frames.len();
        &self.frames[(self.frame as usize) % n]
    }

    /// Advance one tick: move, cycle the frame, and self-kill off-screen.
    pub fn update(&mut self, screen_w: u16, screen_h: u16) {
        self.x += self.dx;
        self.y += self.dy;
        self.frame += self.frame_speed;

        if self.die_offscreen {
            let f = self.current();
            let off_x = self.x + f.width() as f64 <= 0.0 || self.x >= screen_w as f64;
            let off_y = self.y + f.height() as f64 <= 0.0 || self.y >= screen_h as f64;
            if off_x || off_y {
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

/// The color palette digits are randomized into, matching the Perl `rand_color`.
const PALETTE: [char; 12] = ['c', 'C', 'r', 'R', 'y', 'Y', 'b', 'B', 'g', 'G', 'm', 'M'];

/// Resolve a raw fish mask into concrete color letters.
///
/// The Perl assigns one random palette color per digit 1..9 (so every "1" cell
/// gets the same color), after forcing the eye (4) to white. We reproduce that:
/// stable per-fish, random across fish.
pub fn resolve_fish_mask(raw: &str, rng: &mut impl Rng) -> String {
    let mut map = std::collections::HashMap::new();
    map.insert('4', 'W'); // eye is always white
    raw.chars()
        .map(|c| {
            if c.is_ascii_digit() && c != '0' {
                *map.entry(c)
                    .or_insert_with(|| PALETTE[rng.gen_range(0..PALETTE.len())])
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
}
