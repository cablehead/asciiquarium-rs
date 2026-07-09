<h1 align="center">asciiquarium-rs</h1>

<p align="center">
  An aquarium animation in ASCII art, for your terminal -- a Rust port of Kirk
  Baucom's classic Perl
  <a href="https://robobunny.com/projects/asciiquarium/"><code>asciiquarium</code></a>.
</p>

<p align="center">
  <a href="#build-and-run">Build &amp; run</a>
  &middot;
  <a href="#how-the-original-works">How the original works</a>
  &middot;
  <a href="#the-rust-port">The Rust port</a>
  &middot;
  <a href="#credit-and-lineage">Credits</a>
</p>

<p align="center">
  <img alt="license: GPL-2.0-or-later" src="https://img.shields.io/badge/license-GPL--2.0--or--later-blue.svg">
  <img alt="built with crossterm" src="https://img.shields.io/badge/built%20with-crossterm-orange.svg">
</p>

```
         |    |    |                                             . .
        )_)  )_)  )_)                                          '.-:-.`
       )___))___))___)\                                        '  :  '
      )____)____)_____)\\                                   .-----:
    _____|____|____|____\\\__                             .'       `.
    \                   /                           ,    /       (o) \
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\`._/~~~~~~~~~~,__)~~~~~~~~~~~~~
^^^^ ^^^  ^^^   ^^^    ^^^^      ^^^^ ^^^  ^^^   ^^^    ^^^^      ^^^^ ^^^  ^^^   ^^
^^^^      ^^^^     ^^^    ^^     ^^^^      ^^^^     ^^^    ^^     ^^^^      ^^^^
^^      ^^^^      ^^^    ^^^^^^  ^^      ^^^^      ^^^    ^^^^^^  ^^      ^^^^

                 _ _ _                      \'`.
              .='\ \ \`"=,                   )  \
            .'\ \ \ \ \ \ \     (`.      _.-`' ' '`-.
 \'=._     / \ \ \_\_\_\_\_\     \ `.  .`        (o) \_          T~~
 \'=._'.  /\ \,-"`- _ - _ - '-.   >  ><     (((       (          |
   \`=._\|'.\/- _ - _ - _ - _- \ / .`  `._      /_|  /'         /^\
   ;"= ._\=./_ -_ -_ {`"=_    @ (.`       `-. _  _.-`          /   \
    ;="_-_=- _ -  _ - {"=_"-     \          /__/'  _   _   _  /     \  _   _   _
    ;_=_--_.,          {_.='   .-/   \            [ ]_[ ]_[ ]/ _   _ \[ ]_[ ]_[ ]
   ;.="` / ';\        _.     _.-`   / \        O  |_=__-_ =_|_[ ]_[ ]_|_=-___-__|
   /_.='/ \/ /;._ _ _{.-;`/"`     >=_('>       o   | _- =  | =_ = _    |= _=   |
 /._=_.'   '/ / / / /{.= /          \_/        .   |= -[]  |- = _ = /  |_-=_[] |
 /.='       `'./_/_.=`{_/            /      (      | =_    |= - ___/ \ | =_ =  |
                                             )  )  |=  []- |-  /| <')_=<=_ =[] |
                                            (  (   |- =_   | =| | |\_/ |- = -  |
                                             )  )  |_______|__|_|_|_\__|_______|
                                            (  (
```

---

The full cast: fish (two art sets) and their bubbles, seaweed, a castle, and a
rotating headliner -- shark, ship, whale with a spout, sea monster, or big fish.
Pass `-c` for classic mode (the original art only).

## Build and run

```
cargo install asciiquarium-rs   # command is `asciiquarium-rs`
cargo run                       # or run from a checkout
cargo run -- --classic          # classic art set (-c works too)
```

Keys: `q` quit, `p` pause, `r` rebuild. The scene refits when you resize.

The command is `asciiquarium-rs`, not `asciiquarium`, so it won't clash with the
Perl original if you have both.

## Credit and lineage

Almost none of the creative work here is mine.

- **2003** -- `asciiquarium` by **Kirk Baucom**, still at
  [robobunny.com](https://robobunny.com/projects/asciiquarium/). This port
  follows version 1.1.
- **1996-2003** -- most of the art is by
  **[Joan Stark](https://en.wikipedia.org/wiki/Joan_Stark)** (`jgs`, aka
  `spunk1111`) of the Usenet `alt.ascii.art` scene.
- **~2011** -- the "new" fish and sea monster came from the Asciiquarium Live
  Wallpaper for Android, backported to Perl by **Claudio Matsuoka**, whose
  [repo](https://github.com/cmatsuoka/asciiquarium) is the copy this port
  follows.
- **2026** -- this Rust port.

## How the original works

The Perl is mostly art strings and entity declarations. The animation engine is
a separate module, [`Term::Animation`](https://metacpan.org/dist/Term-Animation)
(also Baucom's), built on `Curses`. This port replaces both with one hand-rolled
`crossterm` compositor.

Five ideas carry the whole thing.

**Color masks.** A sprite is a shape plus a same-shaped grid of color codes.
Digits are palette slots, picked at random per spawn, so one drawing yields many
fish:

```
 shape     mask
    \        2
   / \      1 1
 >=_('>    661745
   \_/      111
    /        3

 1 body   2 fin   3 flippers   4 eye (forced white)   5 mouth   6 tail   7 gills
```

**Depth.** A table gives each kind a z; lower z draws on top.

```perl
my %depth = (shark => 2, fish_start => 3, seaweed => 21, castle => 22);
```

**Transparency.** Blanks let lower sprites show through. Space is one; `?` is the
other, which is why the shark is full of them:

```
  ,??????????????????????????)   `\      # ? = water behind shows through
```

**Entities respawn themselves.** Each carries a death callback that adds its
replacement, so the tank never empties:

```perl
death_cb => \&add_fish,   # a fish that leaves spawns another
```

The rotating headliner works the same way, so there's always exactly one.

**One call is both clock and keyboard:**

```perl
halfdelay(1);       # getch() waits at most 0.1s
my $in = getch();   # a key, or nothing -> draw the next frame (~10 fps)
```

## Perl worth a second look

**Art lives in `q{...}` literals**, where `\\` means one backslash and `\}`
means a brace. So the source is doubled up:

```perl
q{
   \\        # a single backslash on screen
  / \\
>=_('>
}
```

Re-typing that by hand is a trap, so the port lets Perl unescape its own strings
(see [`tools/`](tools/)).

**A bug, kept on purpose.** The color picker is off by one:

```perl
my @c = ('c','C','r','R','y','Y','b','B','g','G','m','M');  # 12 colors
$c[ int(rand($#c)) ];   # $#c is 11, not 12 -- so 'M' is never chosen
```

The port reproduces it, so the brightest magenta never shows up, just like the
original.

## The Rust port

No crate matches `Term::Animation`, so the engine is hand-rolled -- a few
hundred lines across five files:

- **`render.rs`** -- the compositor. Sprites blit into a `Vec<Cell>` sorted by
  depth, skipping transparent cells; one write to the terminal per frame.
- **`entity.rs`** -- sprites: multi-frame shape+mask, fractional velocity, three
  ways to die (off-screen, frame count, deadline).
- **`art.rs`** -- the art, generated byte-exact from the Perl by
  [`tools/`](tools/). **`spawn.rs`** turns it into creatures and ports each
  `add_*`.
- **`main.rs`** -- the loop. `poll(100ms)` is clock and input at once, like
  `halfdelay`; resize and collisions live here.

Two things Rust handles more cleanly than the original:

- **Respawns don't alias.** The Perl edits the live scene from inside a
  callback. The port collects new spawns and appends them after the frame
  instead.
- **The terminal always resets.** A `Drop` guard restores raw mode on any exit
  -- normal, error, or panic -- replacing the Perl's trap-every-signal habit.

## Differences from the original

Timing is tick-based, not wall-clock. Movement, frame changes, and lifetimes all
advance once per loop, which `poll(100ms)` holds near 10 fps. This matches
`Term::Animation`, so the feel is the same -- but it isn't frame-rate
independent: run the loop faster or slower and the whole tank does too.

One bonus: the original needs curses and won't run on Windows; `crossterm` is
cross-platform.

## License

**GPL-2.0-or-later**, same as the original -- see [`LICENSE`](LICENSE).

This is a derivative work: it copies the original art verbatim, so it inherits
the license and can't be relicensed. Copyright in the art and program stays with
Kirk Baucom, Joan Stark, and Claudio Matsuoka; this port adds only the Rust code,
under the same terms.
