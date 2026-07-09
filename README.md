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

The full cast: fish ([two art sets](#credit-and-lineage)) and their bubbles,
seaweed, a castle, and a rotating headliner -- shark, ship, whale with a spout, sea monster, or big fish.
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

Almost none of the creative work is mine.

- **Kirk Baucom** wrote `asciiquarium` (2003) in Perl, and the engine under it,
  `Term::Animation`. Still online at
  [robobunny.com](https://robobunny.com/projects/asciiquarium/).
- **[Joan Stark](https://en.wikipedia.org/wiki/Joan_Stark)** (`jgs`, aka
  `spunk1111`) drew most of the art, during her run on Usenet's `alt.ascii.art`
  from 1996 to 2003.
- **Claudio Matsuoka** added the "new" fish and sea monster. They first appeared
  in his [Asciiquarium Live Wallpaper for Android](https://archive.org/details/org.helllabs.android.asciiquarium)
  (2011); he then backported them into the Perl.

This port follows Matsuoka's canonical copy,
[cmatsuoka/asciiquarium](https://github.com/cmatsuoka/asciiquarium) -- which is
version 1.1.

## How the original works

The striking thing about the Perl is how little of it is animation code. Of its
~1500 lines, almost all are art strings and entity declarations; the engine
itself is a separate module,
[`Term::Animation`](https://metacpan.org/dist/Term-Animation) (also Baucom's),
sitting on `Curses`. The script is really a scene description -- and this port
collapses both layers, engine and curses, into one hand-rolled `crossterm`
compositor.

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
  `halfdelay`. Resizing refits the scene live -- the original only rebuilds when
  you press `r`. Collisions run here too.

Two things Rust handles more cleanly than the original:

- **Respawns don't alias.** The Perl edits the live scene from inside a
  callback. The port collects new spawns and appends them after the frame
  instead.
- **The terminal always resets.** A `Drop` guard restores raw mode on any exit
  -- normal, error, or panic -- replacing the Perl's trap-every-signal habit.

Built on `crossterm` rather than curses, it also runs on Windows, which the
original can't.

## License

**GPL-2.0-or-later**, same as the original -- see [`LICENSE`](LICENSE).

This is a derivative work: it copies the original art verbatim, so it inherits
the license and can't be relicensed. Copyright in the art and program stays with
Kirk Baucom, Joan Stark, and Claudio Matsuoka; this port adds only the Rust code,
under the same terms.
