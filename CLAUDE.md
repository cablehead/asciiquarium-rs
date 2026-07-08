## Git Commit Style Preferences

**NEVER commit unless explicitly asked by the user.**

When committing: review `git diff`

- Use conventional commit format: `type: subject line`
- Keep subject line concise and descriptive
- **NEVER include marketing language, promotional text, or AI attribution**
- **NEVER add "Generated with Claude Code", "Co-Authored-By: Claude", or similar spam**
- Follow existing project patterns from git log
- Prefer just a subject and no body, unless the change is particularly complex

Example good commit messages:
- `feat: add shark and its collision teeth`
- `fix: bubbles now pop at the correct waterline row`
- `refactor: fold color-mask resolution into the Frame constructor`

## Tone and Communication

- ASCII only. No em dashes, smart quotes, or other unicode punctuation. Use "--" only in code contexts, not as prose punctuation.
- No wasted words. No fluff. Each word should add value to the reader.
- Calm, matter-of-fact technical tone.

## Code Quality

Always run `./scripts/check.sh` before committing. Use `cargo fmt` to fix formatting issues.

## About This Project

A Rust port of the classic `asciiquarium` Perl script (see README.md for full
attribution). The Perl leans entirely on the `Term::Animation` CPAN module;
there is no Rust equivalent, so the engine is hand-rolled here:

- `src/render.rs` -- the compositor: a `Vec<Cell>` shadow buffer, z-sorted
  painter's-algorithm blit with space-transparency, flushed once per frame.
- `src/entity.rs` -- the entity model: multi-frame sprites, parallel color
  masks, fractional per-tick velocity, off-screen death.
- `src/art.rs` -- ASCII art transcribed from the original. Use raw strings
  (`r"..."`); re-check every backslash against a running copy of the Perl.
- `src/main.rs` -- CLI (clap), the crossterm event loop (which is also the
  frame clock), and the `add_*`-style spawners.

### Porting notes

- The original's shape and color-mask arrays are index-coupled; keep them
  together in a single `Frame` here.
- `death_cb` in the Perl re-spawns replacements by mutating the animation
  mid-iteration. Here, `tick()` collects spawn requests into a `Vec` and appends
  them after the pass -- do not mutate the entity list while iterating it.
- Lower z draws on top (painter's algorithm sorts z descending). This matches
  the original's `%depth` semantics; do not invert it.
