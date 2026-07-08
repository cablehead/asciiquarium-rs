//! ASCII art, transcribed from the original asciiquarium.
//!
//! Perl wrote these as `q{...}` / `q#...#` literals; Rust raw strings (`r"..."`)
//! serve the same role but without Perl's brace/backslash escaping, so the art
//! reads exactly as it renders. This prototype carries one fish (both facings)
//! to prove the shape/mask/z/transparency model; the remaining spawners port in
//! the same shape.

/// The four tiling segments of the water surface, top to bottom.
pub const WATER_SEGMENTS: [&str; 4] = [
    "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~",
    "^^^^ ^^^  ^^^   ^^^    ^^^^      ",
    "^^^^      ^^^^     ^^^    ^^     ",
    "^^      ^^^^      ^^^    ^^^^^^  ",
];

/// Bubble growth frames, smallest to largest (rises then pops at the surface).
pub const BUBBLE: [&str; 5] = [".", "o", "O", "O", "O"];

/// A small fish, facing right. Fields index into the color mask:
/// 1 body, 2 dorsal fin, 3 flippers, 4 eye, 5 mouth, 6 tailfin, 7 gills.
pub const FISH_R_SHAPE: &str = r"
   \
  / \
>=_('>
  \_/
   /
";

pub const FISH_R_MASK: &str = r"
   2
  1 1
661745
  111
   3
";

/// The same fish, facing left.
pub const FISH_L_SHAPE: &str = r"
  /
 / \
<')_=<
 \_/
  \
";

pub const FISH_L_MASK: &str = r"
  2
 1 1
547166
 111
  3
";
