<h1 align="center">asciiquarium-rs</h1>

<p align="center">
  An aquarium animation in ASCII art for your terminal. A Rust port of Kirk
  Baucom's classic Perl
  <a href="https://robobunny.com/projects/asciiquarium/"><code>asciiquarium</code></a>
  (2003).
</p>

<p align="center">
  <a href="#build-and-run">Build &amp; run</a>
  &middot;
  <a href="#credit-and-lineage">Credits</a>
  &middot;
  <a href="#how-the-original-works">How the original works</a>
  &middot;
  <a href="#the-rust-port">The Rust port</a>
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

The full cast: fish ([both art sets](#credit-and-lineage)) and their bubbles,
seaweed, a castle, and a rotating headliner: shark, ship, whale with a spout,
sea monster, or big fish. Pass `-c` for classic mode (the original art only).

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
[cmatsuoka/asciiquarium](https://github.com/cmatsuoka/asciiquarium), which is
version 1.1.

## How the original works

The striking thing about the Perl is how little of it is animation code. Of its
~1500 lines, almost all are art strings and entity declarations; the engine
itself is a separate module,
[`Term::Animation`](https://metacpan.org/dist/Term-Animation) (also Baucom's),
sitting on `Curses`. The script is really a scene description.

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

**Transparency.** Two characters don't draw, letting lower sprites through: space
and `?` (which is why the shark is full of them):

```
  ,??????????????????????????)   `\      # ? = water behind shows through
```

**Entities respawn themselves.** Each carries a death callback that adds its
replacement, so the tank never empties:

```perl
death_cb => \&add_fish,   # a fish that leaves spawns another
```

The rotating headliner works the same way, so there's always exactly one.

**One call is both clock and keyboard** ([`halfdelay`](https://github.com/cmatsuoka/asciiquarium/blob/8bdb7d441a36a5a9f64b853317a66f9d4a82f08f/asciiquarium#L102)):

```perl
halfdelay(1);       # getch() waits at most 0.1s
my $in = getch();   # a key, or nothing -> draw the next frame (~10 fps)
```

## The Rust port

No crate matches `Term::Animation`, so the engine is hand-rolled, a few hundred
lines.

**The compositor** does what `Term::Animation` and `Curses` did together: sprites
blit into a flat cell buffer, back-to-front by depth, skipping transparent
cells, and it flushes to the terminal once per frame.

**The art is generated, not retyped.** The original stores its sprites right in
the source, as `q{...}` string literals. A backslash escapes even there, so
every `\` in the art is written `\\`:

```perl
q{
   \\        # renders as one backslash
  / \\
>=_('>
}
```

Copying those verbatim would double every backslash. So instead a script in
[`tools/`](tools/) has Perl eval its own literals and emit them as Rust
constants, byte for byte.

**The loop** is one `poll(100ms)`, serving as clock and keyboard at once, like
`halfdelay`. It refits the scene the moment you resize, where the original only
rebuilds on `r`.

The port stays faithful down to the quirks: `rand($#c)` indexes with the last
position (11) of a 12-color list rather than the count (12), so one color never
appears in the original, and it doesn't here either.

And on `crossterm` rather than `Curses`, it runs on Windows, which the original
can't.

## License

**GPL-2.0-or-later**, same as the original. See [`LICENSE`](LICENSE).

This is a derivative work: it copies the original art verbatim, so it inherits
the license and can't be relicensed. Copyright in the art and program stays with
Kirk Baucom, Joan Stark, and Claudio Matsuoka; this port adds only the Rust code,
under the same terms.
