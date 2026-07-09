//! The compositor: a shadow buffer of colored cells that entities blit into,
//! flushed to the terminal once per frame.
//!
//! This is the piece Perl's Term::Animation gave us for free. It is small: a
//! flat `Vec<Cell>`, a painter's-algorithm blit that honors space-transparency,
//! and a single write to stdout per frame.

use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{Color, Print, SetForegroundColor},
};

/// One terminal cell: a glyph and its foreground color.
#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            ch: ' ',
            fg: Color::Reset,
        }
    }
}

/// A width x height grid of cells.
pub struct Screen {
    pub w: u16,
    pub h: u16,
    cells: Vec<Cell>,
}

impl Screen {
    pub fn new(w: u16, h: u16) -> Self {
        Screen {
            w,
            h,
            cells: vec![Cell::default(); w as usize * h as usize],
        }
    }

    /// Reset every cell to blank. Called at the top of each frame.
    pub fn clear(&mut self) {
        for c in &mut self.cells {
            *c = Cell::default();
        }
    }

    /// Blit one sprite frame at (x, y). `shape` and `mask` are line-aligned:
    /// the mask char at the same row/col picks the color, falling back to
    /// `default` where the mask is blank. A shape char is transparent when it
    /// equals `trans` (or is a space and `auto_trans` is set), letting whatever
    /// a lower-z entity wrote show through. This is the whole trick behind
    /// fish swimming "in front of" the castle.
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &mut self,
        x: i32,
        y: i32,
        shape: &[String],
        mask: &[String],
        default: Color,
        auto_trans: bool,
        trans: char,
    ) {
        for (row, line) in shape.iter().enumerate() {
            let cy = y + row as i32;
            if cy < 0 || cy >= self.h as i32 {
                continue;
            }
            let mask_line = mask.get(row).map(|s| s.as_str()).unwrap_or("");
            for (col, ch) in line.chars().enumerate() {
                if ch == trans || (auto_trans && ch == ' ') {
                    continue; // transparent
                }
                let cx = x + col as i32;
                if cx < 0 || cx >= self.w as i32 {
                    continue;
                }
                let fg = mask_line
                    .chars()
                    .nth(col)
                    .and_then(mask_color)
                    .unwrap_or(default);
                let idx = cy as usize * self.w as usize + cx as usize;
                self.cells[idx] = Cell { ch, fg };
            }
        }
    }

    /// The buffer as plain text (no color), rows joined by newlines. Handy for
    /// snapshot tests and debugging.
    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity((self.w as usize + 1) * self.h as usize);
        for row in 0..self.h {
            for col in 0..self.w {
                out.push(self.cells[row as usize * self.w as usize + col as usize].ch);
            }
            if row + 1 < self.h {
                out.push('\n');
            }
        }
        out
    }

    /// Flush the buffer to stdout in one pass, emitting a color escape only when
    /// the run color changes.
    pub fn render(&self, out: &mut impl Write) -> io::Result<()> {
        queue!(out, cursor::MoveTo(0, 0))?;
        let mut cur = Color::Reset;
        for row in 0..self.h {
            let mut line = String::with_capacity(self.w as usize);
            for col in 0..self.w {
                let cell = self.cells[row as usize * self.w as usize + col as usize];
                if cell.fg != cur {
                    if !line.is_empty() {
                        queue!(out, Print(std::mem::take(&mut line)))?;
                    }
                    queue!(out, SetForegroundColor(cell.fg))?;
                    cur = cell.fg;
                }
                line.push(cell.ch);
            }
            queue!(out, Print(line))?;
            if row + 1 < self.h {
                queue!(out, Print("\r\n"))?;
            }
        }
        out.flush()
    }
}

/// Map an asciiquarium color-mask letter to a terminal color. Lowercase is the
/// dim variant, uppercase the bright one, matching the original curses scheme.
pub fn mask_color(c: char) -> Option<Color> {
    Some(match c {
        'c' => Color::DarkCyan,
        'C' => Color::Cyan,
        'r' => Color::DarkRed,
        'R' => Color::Red,
        'y' => Color::DarkYellow,
        'Y' => Color::Yellow,
        'b' => Color::DarkBlue,
        'B' => Color::Blue,
        'g' => Color::DarkGreen,
        'G' => Color::Green,
        'm' => Color::DarkMagenta,
        'M' => Color::Magenta,
        'w' => Color::Grey,
        'W' => Color::White,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell_at(s: &Screen, x: u16, y: u16) -> Cell {
        s.cells[y as usize * s.w as usize + x as usize]
    }

    #[test]
    fn space_is_transparent_with_auto_trans() {
        let mut s = Screen::new(4, 1);
        s.blit(0, 0, &["XX".into()], &[], Color::Reset, true, '?');
        // A leading space with auto_trans must not overwrite the first cell.
        s.blit(0, 0, &[" Y".into()], &[], Color::Reset, true, '?');
        assert_eq!(cell_at(&s, 0, 0).ch, 'X');
        assert_eq!(cell_at(&s, 1, 0).ch, 'Y');
    }

    #[test]
    fn question_mark_transparent_but_space_opaque_without_auto_trans() {
        let mut s = Screen::new(3, 1);
        s.blit(0, 0, &["XXX".into()], &[], Color::Reset, false, '?');
        // Without auto_trans, '?' is transparent but ' ' overwrites.
        s.blit(0, 0, &["? Z".into()], &[], Color::Reset, false, '?');
        assert_eq!(cell_at(&s, 0, 0).ch, 'X'); // kept (transparent '?')
        assert_eq!(cell_at(&s, 1, 0).ch, ' '); // overwritten (opaque space)
        assert_eq!(cell_at(&s, 2, 0).ch, 'Z');
    }

    #[test]
    fn mask_selects_color_else_default() {
        let mut s = Screen::new(2, 1);
        s.blit(0, 0, &["ab".into()], &["R".into()], Color::Green, true, '?');
        assert_eq!(cell_at(&s, 0, 0).fg, Color::Red); // masked
        assert_eq!(cell_at(&s, 1, 0).fg, Color::Green); // default
    }
}
