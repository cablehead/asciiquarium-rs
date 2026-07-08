# asciiquarium-rs

An aquarium/sea animation in ASCII art, for your terminal -- a Rust port of the
classic Perl `asciiquarium`.

```
   \                                            /
  / \       >=_('>                    <')_=<   / \
>=_('>                     .                  <')_=<
  \_/         o           ( )                  \_/
   /                                            \
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
```

## Status

Early prototype. This is a deliberately narrow vertical slice that proves the
rendering model: a hand-rolled compositor, the entity system, color masks, and
the event loop, with fish swimming and blowing bubbles beneath a tiled water
surface. The rest of the original's cast (seaweed, castle, sharks, ships,
whales, sea monsters, big fish) ports onto these same pieces -- see the roadmap.

## Credit and lineage

This is a port; essentially none of the creative work is mine.

- Original `asciiquarium` by **Kirk Baucom** -- <https://robobunny.com/projects/asciiquarium/>
- Most of the ASCII art by **Joan Stark** (`spunk1111`), a giant of the ASCII
  art scene.
- Expanded marine biodiversity (the "new" fish and sea monster) by
  **Claudio Matsuoka**, backported from the Asciiquarium Live Wallpaper for
  Android. The reference implementation this port follows is Claudio's:
  **<https://github.com/cmatsuoka/asciiquarium>**

The original is licensed GPL-2.0-or-later, and so is this port.

## Build and run

```
cargo run              # swim
cargo run -- --classic # classic mode: original art set only (-c also works)
```

Controls: `q` quit, `p` pause, `r` rebuild the scene. Resize the terminal and
the scene rebuilds to fit.

## How the original pulls it off

Reading the Perl, the surprising thing is how little of it is animation code.
Almost the entire ~1500 lines are two things: piles of ASCII-art strings, and
entity declarations. The actual engine lives in a CPAN module,
`Term::Animation`, which the script drives. So the program is really a
*scene description*, and the interesting mechanics are in how that description
is encoded. A few highlights:

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
  velocity so fish move at sub-cell speeds, and off-screen death. `resolve_fish_mask`
  reproduces the Perl's per-digit random coloring exactly, eye-forced-white and
  all.

- **`src/main.rs` -- the loop.** `crossterm::event::poll(100ms)` is the direct
  analog of `halfdelay(1)`: one call that is both the input read and the frame
  clock. `Event::Resize` comes through the same stream, so live resize is free
  -- an improvement over the original, which ignores `SIGWINCH` and only reacts
  on `r`.

- **`death_cb` without shared mutable state.** The Perl mutates the live
  animation from inside a callback mid-iteration. Rust's borrow checker would
  fight that, and it is genuinely error-prone, so `tick()` instead collects
  spawn requests into a `Vec` during the pass and appends them after -- same
  behavior, no aliasing.

- **A `TerminalGuard` with a `Drop` impl** restores raw mode and the alternate
  screen no matter how the program leaves -- normal exit, `?` early-return, or
  panic. That is the structural version of the original's install-every-signal
  paranoia: you cannot forget to clean up, because cleanup is tied to the value
  going out of scope.

- **Windows support, incidentally.** The original notes it needs a curses
  library and so will not run on Windows. `crossterm` is cross-platform, so this
  port is not so limited.

## Roadmap

- [x] Compositor: z-depth, space-transparency, color masks
- [x] Entities: multi-frame sprites, fractional movement, off-screen death
- [x] Fish (both facings), bubbles, tiled waterline
- [x] `-c` / `--classic` flag plumbed through
- [ ] Seaweed (swaying, timed lifecycle) and the castle
- [ ] Collisions: shark teeth eating small fish (with a splat), bubbles popping
      handled generically rather than the current waterline special-case
- [ ] The roaming specials: ship, whale (with spout animation), shark, sea
      monster, big fish
- [ ] The full "new" art sets vs. classic
