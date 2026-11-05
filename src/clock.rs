//! The optional background clock: local time in a seven-row block font, or a
//! five-row line font when the big one does not fit. Spaces are transparent,
//! so fish swim through the gaps. In 12-hour mode a three-row AM/PM sits at
//! the bottom right, like a digital clock's indicator.

use chrono::Timelike;

/// Small font glyph rows for 0-9. Each digit is five columns wide.
const SMALL_DIGITS: [[&str; 5]; 10] = [
    [" ___ ", "|   |", "|   |", "|   |", "|___|"],
    ["     ", "    |", "    |", "    |", "    |"],
    [" ___ ", "    |", " ___|", "|    ", "|___ "],
    [" ___ ", "    |", " ___|", "    |", " ___|"],
    ["     ", "|   |", "|___|", "    |", "    |"],
    [" ___ ", "|    ", "|___ ", "    |", " ___|"],
    [" ___ ", "|    ", "|___ ", "|   |", "|___|"],
    [" ___ ", "    |", "    |", "    |", "    |"],
    [" ___ ", "|   |", "|___|", "|   |", "|___|"],
    [" ___ ", "|   |", "|___|", "    |", " ___|"],
];

const SMALL_COLON: [&str; 5] = ["   ", " o ", "   ", " o ", "   "];

/// AM/PM letters, drawn in both sizes and bottom-aligned with the digits.
fn suffix_letter(c: char) -> Option<[&'static str; 3]> {
    Some(match c {
        'A' => [" _ ", "|_|", "| |"],
        'P' => [" _ ", "|_|", "|  "],
        'M' => ["    ", "|\\/|", "|  |"],
        _ => return None,
    })
}

/// Lit segments per digit, as "abcdefg" bits: a top, b upper right, c lower
/// right, d bottom, e lower left, f upper left, g middle.
const SEGMENTS: [u8; 10] = [
    0b1111110, 0b0110000, 0b1101101, 0b1111001, 0b0110011, //
    0b1011011, 0b1011111, 0b1110000, 0b1111111, 0b1111011,
];

const BIG_W: usize = 7;
const BIG_H: usize = 7;
const BLOCK: char = '#';

/// One big digit: full-width bars, two-column verticals.
fn big_digit(n: usize) -> Vec<String> {
    let on = |seg: u8| SEGMENTS[n] & (1 << (6 - seg)) != 0; // seg 0 = a
    let (mid, last) = (BIG_H / 2, BIG_H - 1);
    (0..BIG_H)
        .map(|r| {
            (0..BIG_W)
                .map(|c| {
                    let bar = (r == 0 && on(0)) || (r == mid && on(6)) || (r == last && on(3));
                    let left = c < 2 && ((r <= mid && on(5)) || (r >= mid && on(4)));
                    let right = c >= BIG_W - 2 && ((r <= mid && on(1)) || (r >= mid && on(2)));
                    if bar || left || right {
                        BLOCK
                    } else {
                        ' '
                    }
                })
                .collect()
        })
        .collect()
}

fn big_colon() -> Vec<String> {
    (0..BIG_H)
        .map(|r| {
            let c = if r == 2 || r == 4 { BLOCK } else { ' ' };
            c.to_string().repeat(2)
        })
        .collect()
}

fn small_glyph(rows: &[&str; 5]) -> Vec<String> {
    rows.iter().map(|r| r.to_string()).collect()
}

/// An AM/PM letter padded with blank rows on top to `height`.
fn suffix_glyph(rows: [&str; 3], height: usize) -> Vec<String> {
    let blank = " ".repeat(rows[0].len());
    std::iter::repeat_n(blank, height - rows.len())
        .chain(rows.iter().map(|r| r.to_string()))
        .collect()
}

/// Lay glyphs side by side, `gap` spaces apart. A space is an empty glyph, so
/// it doubles the gap. Other unknown characters are skipped.
fn layout(text: &str, big: bool) -> Vec<String> {
    let (height, gap) = if big { (BIG_H, "  ") } else { (5, " ") };
    let glyphs: Vec<Vec<String>> = text
        .chars()
        .filter_map(|c| match (c, big) {
            (' ', _) => Some(vec![String::new(); height]),
            (':', true) => Some(big_colon()),
            (':', false) => Some(small_glyph(&SMALL_COLON)),
            (c, _) if suffix_letter(c).is_some() => {
                suffix_letter(c).map(|rows| suffix_glyph(rows, height))
            }
            (d, true) => d.to_digit(10).map(|n| big_digit(n as usize)),
            (d, false) => d
                .to_digit(10)
                .map(|n| small_glyph(&SMALL_DIGITS[n as usize])),
        })
        .collect();
    (0..height)
        .map(|row| {
            glyphs
                .iter()
                .map(|g| g[row].as_str())
                .collect::<Vec<_>>()
                .join(gap)
        })
        .collect()
}

/// Render `text` as a shape block: the block font when it fits within
/// `max_w` x `max_h`, else the small font.
pub fn shape(text: &str, max_w: i32, max_h: i32) -> String {
    let big = layout(text, true);
    let fits = big.len() as i32 <= max_h && big[0].chars().count() as i32 <= max_w;
    if fits { big } else { layout(text, false) }.join("\n")
}

/// Format a time as `H:MM AM` (12-hour, no leading zero) or `HH:MM` (24-hour).
pub fn text(hour: u32, minute: u32, twelve_hour: bool) -> String {
    if twelve_hour {
        let h = match hour % 12 {
            0 => 12,
            h => h,
        };
        let suffix = if hour < 12 { "AM" } else { "PM" };
        format!("{h}:{minute:02} {suffix}")
    } else {
        format!("{hour:02}:{minute:02}")
    }
}

/// The current local time, formatted by `text`.
pub fn now_text(twelve_hour: bool) -> String {
    let now = chrono::Local::now();
    text(now.hour(), now.minute(), twelve_hour)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_both_hour_styles() {
        assert_eq!(text(0, 5, false), "00:05");
        assert_eq!(text(23, 59, false), "23:59");
        assert_eq!(text(0, 5, true), "12:05 AM");
        assert_eq!(text(11, 59, true), "11:59 AM");
        assert_eq!(text(12, 0, true), "12:00 PM");
        assert_eq!(text(13, 7, true), "1:07 PM");
    }

    #[test]
    fn shape_rows_are_aligned() {
        let rows = |s: String| s.lines().map(|r| r.chars().count()).collect::<Vec<_>>();
        // Four digits, one colon, four gaps.
        assert_eq!(rows(shape("12:34", 100, 100)), vec![4 * 7 + 2 + 4 * 2; 7]);
        assert_eq!(rows(shape("12:34", 30, 100)), vec![4 * 5 + 3 + 4; 5]);
        assert_eq!(rows(shape("12:34", 100, 6)), vec![4 * 5 + 3 + 4; 5]);
    }

    #[test]
    fn suffix_sits_at_the_bottom_right() {
        for (max_w, rows, gap) in [(100, 7, "  "), (40, 5, " ")] {
            let s = shape("1:07 PM", max_w, 100);
            let lines: Vec<&str> = s.lines().collect();
            assert_eq!(lines.len(), rows);
            // " _ " of the P sits three rows from the bottom, blank above it.
            let p_col = lines[0].len() - 3 - gap.len() - 4;
            assert_eq!(&lines[rows - 3][p_col..p_col + 3], " _ ");
            assert!(lines[..rows - 3]
                .iter()
                .all(|l| l[p_col..].trim().is_empty()));
            assert!(lines[rows - 1].ends_with(&format!("|  {gap}|  |")));
        }
    }

    #[test]
    fn big_eight_is_a_filled_frame() {
        let eight = big_digit(8);
        assert_eq!(eight[0], "#######");
        assert_eq!(eight[1], "##   ##");
        assert_eq!(eight[3], "#######");
        assert_eq!(eight[6], "#######");
        assert_eq!(big_digit(1)[0], "     ##");
    }

    #[test]
    fn glyphs_are_ascii_and_uniform() {
        for g in SMALL_DIGITS.iter().chain(std::iter::once(&SMALL_COLON)) {
            let w = g[0].len();
            assert!(g.iter().all(|r| r.len() == w && r.is_ascii()));
        }
        for c in ['A', 'P', 'M'] {
            let g = suffix_letter(c).unwrap();
            assert!(g.iter().all(|r| r.len() == g[0].len() && r.is_ascii()));
        }
    }
}
