# asciiquarium-rs

An aquarium/sea animation in ASCII art, for your terminal -- a Rust port of the
classic Perl `asciiquarium`, written by Kirk Baucom in 2003
([robobunny.com](https://robobunny.com/projects/asciiquarium/)). This port
follows Claudio Matsuoka's canonical Perl copy
([cmatsuoka/asciiquarium](https://github.com/cmatsuoka/asciiquarium)), which
adds the later "new" creatures.

```
   \                                            /
  / \       >=_('>                    <')_=<   / \
>=_('>                     .                  <')_=<
  \_/         o           ( )                  \_/
   /                                            \
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
```

The whole cast is here: the water surface, the castle, swaying seaweed, both
fish art sets and their air bubbles, sharks (with the biting "teeth" and a
splat), ships, whales with an animated spout, sea monsters (new and classic),
and both big fish -- all fed by the roaming-special chain, with a `-c` classic
mode. It matches the reference; the handful of intentional differences are noted
at the end.

## Build and run

```
cargo run               # swim
cargo run -- --classic  # classic mode: original art set only (-c also works)
```

Controls: `q` quit, `p` pause, `r` rebuild the scene. Resize the terminal and
the scene rebuilds to fit.

## Credit and lineage

This is a port; essentially none of the creative work is mine.

- **2003** -- `asciiquarium` created by **Kirk Baucom**, and still maintained at
  <https://robobunny.com/projects/asciiquarium/>. Version 1.1 (2013) is the one
  ported here.
- **late 1990s** -- most of the ASCII art is by **Joan Stark** (`spunk1111`), a
  giant of the ASCII art scene whose work dates from the height of the web-art
  era.
- **~2011** -- the extra "new" fish and sea monster were drawn for the
  **Asciiquarium Live Wallpaper for Android**, then backported into the Perl by
  **Claudio Matsuoka**, whose GitHub repo became the canonical copy this port
  follows: <https://github.com/cmatsuoka/asciiquarium>
- **2026** -- this Rust port.

The original is licensed GPL-2.0-or-later, and so is this port.

## How the original pulls it off

Reading the Perl, the surprising thing is how little of it is animation code.
Almost the entire ~1500 lines are two things: piles of ASCII-art strings, and
entity declarations. The actual engine lives in a CPAN module,
[`Term::Animation`](https://metacpan.org/dist/Term-Animation) -- which Kirk
Baucom wrote himself (CPAN id `KBAUCOM`), factoring the sprite machinery out of
his own aquarium into a reusable "ASCII sprite animation framework" first
released in 2003 and last updated (v2.6) in 2011. It in turn requires `Curses`,
the XS binding to the C curses library -- which is why the original only runs
where curses exists. So the script is really a *scene description* driving
Baucom's engine, and the interesting mechanics are in how that description is
encoded. (This port therefore replaces two layers at once: Term::Animation's
sprite/z-depth/collision model and curses underneath it, both collapsed into the
hand-rolled `crossterm` compositor.) A few highlights:

- **Color masks.** Every sprite has a shape and a second text block of the same
  layout -- the mask -- where each character picks the color of the shape cell
  at the same row/column. The fish mask uses digits as a numbered palette:
  `1` body, `2` dorsal fin, `3` flippers, `4` eye, `5` mouth, `6` tailfin,
  `7` gills. At spawn time each digit is swapped for a random color, so every
  fish is uniquely colored from the same art. The eye is always forced white.

- **Depth as a number, painter's algorithm.** A `%depth` table assigns every
  kind of object a z value; lower z draws on top. Bubbles sit one z above their
  fish so they always render in front; fish occupy a whole band of z values so
  they overlap plausibly.

- **Transparency for free.** Spaces in a sprite do not overwrite what is behind
  them, so a fish drawn over the castle just works without any per-sprite
  masking beyond the shape itself.

- **Lifecycles via callbacks.** Entities can die off-screen, after a frame
  count, or at a wall-clock time, and each carries a `death_cb` that spawns its
  own replacement. That is the whole trick to a screen that stays populated
  forever: a fish that swims off the edge calls `add_fish` on its way out. The
  single roaming "special" (ship / whale / shark / ...) works the same way --
  its death callback picks a new random special, so there is always exactly one.

- **The frame clock is the input reader.** `halfdelay(1)` makes `getch()` block
  for at most a tenth of a second. That one call both reads a keystroke and
  paces the animation to ~10fps: if no key arrives in 100ms, the loop just
  advances a frame.

## Interesting Perl-isms encountered

- **`q{...}` and `q#...#` literals.** The art is written in Perl's
  single-quote-style literals with arbitrary delimiters. Inside them `\\` is a
  literal backslash and `\}` escapes the delimiter -- which matters, because the
  fish art is *full* of backslashes. Getting the art across faithfully is the
  single biggest correctness hazard in the port.

- **`$#array` vs `scalar(@array)`.** The color picker reads
  `$colors[int(rand($#colors))]`. `$#colors` is the last *index* (11), not the
  count (12), so `rand` never reaches the final palette entry -- the brightest
  magenta is silently unreachable. A tiny latent bug preserved in amber.

- **Parallel arrays coupled by index.** Shapes and masks live in separate
  arrays, paired by `$i` and `$i+1`. It works, but it is fragile; the port folds
  each pair into one struct.

- **`?` is transparency, not a glyph.** The shark, sea monster, and big fish art
  is speckled with `?`. It never renders: `?` is Term::Animation's default
  transparency character, so those cells let the water behind show through.
  Entities with `auto_trans` additionally treat spaces as transparent. Miss this
  and every shark swims inside a box of question marks.

- **Signal paranoia.** The original installs a handler on *every* signal it can,
  specifically so a stray signal can never leave your terminal in raw mode. The
  Rust port gets the same guarantee structurally (see below).

## The Rust port

No crate mirrors `Term::Animation` (the nearest, `gemini-engine`, is a general
ASCII renderer with a different model), so the engine is hand-rolled. It is
small and the interesting parts are:

- **`src/render.rs` -- the compositor.** A flat `Vec<Cell>` shadow buffer. Each
  frame: clear it, blit every entity back-to-front (z sorted descending), and
  flush to stdout in a single pass that emits a color escape only when the run
  color changes. Space-transparency and the shape/mask alignment live here, in
  about 40 lines of `blit`.

- **`src/entity.rs` -- the entity model.** Multi-frame sprites (a `Frame` bundles
  shape + mask so they can never drift out of sync), fractional per-tick
  velocity so fish move at sub-cell speeds, three ways to die (off-screen, a
  frame count, a deadline tick), and a `?`/`auto_trans` transparency test.
  `resolve_fish_mask` reproduces the Perl's per-digit random coloring exactly,
  eye-forced-white and off-by-one bug and all.

- **`src/art.rs` -- generated, byte-exact art.** Rather than retype the
  backslash-dense art by hand, a small Perl script (`tools/`) re-reads the
  reference and evals each `q{...}` / `"..."` literal so Perl itself does the
  de-escaping, then emits this file as raw-string constants.
  `src/spawn.rs` groups those into creatures and ports each `add_*` spawner.

- **`src/main.rs` -- the loop and collisions.** `crossterm::event::poll(100ms)`
  is the direct analog of `halfdelay(1)`: one call that is both the input read
  and the frame clock. `Event::Resize` comes through the same stream, so live
  resize is free -- an improvement over the original, which ignores `SIGWINCH`
  and only reacts on `r`. Collisions are a per-frame cell-overlap pass over the
  physical entities (bubbles vs. the waterline, small fish vs. shark teeth).

- **`death_cb` without shared mutable state.** The Perl mutates the live
  animation from inside a callback mid-iteration. Rust's borrow checker would
  fight that, and it is genuinely error-prone, so `advance()` instead collects
  spawn requests into a `Vec` during the pass and appends them after -- same
  behavior, no aliasing. The single roaming special keeps itself alive this
  way: each one's death spawns the next.

- **A `TerminalGuard` with a `Drop` impl** restores raw mode and the alternate
  screen no matter how the program leaves -- normal exit, `?` early-return, or
  panic. That is the structural version of the original's install-every-signal
  paranoia: you cannot forget to clean up, because cleanup is tied to the value
  going out of scope.

## Differences from the original

Two intentional departures, both about timing:

- Movement is per-tick, paced by the ~10fps `poll` timeout, rather than
  wall-clock scaled. This matches how Term::Animation advances on each
  `animate()` call, so the feel is the same, but it is not frame-rate
  independent.
- The seaweed lifetime is converted from the original's wall-clock seconds to
  ticks, assuming ten per second.

And one incidental gain: the original needs a curses library and so does not run
on Windows, whereas `crossterm` is cross-platform.

## Regenerating the art

`src/art.rs` is generated from the reference Perl, not hand-written. See
[`tools/`](tools/) to reproduce it.
