# tools

Art extraction for the port. `src/art.rs` is generated, not hand-written -- the
original art is backslash-dense and error-prone to retype, so we let Perl
de-escape its own literals.

Point these at a checkout of the reference Perl
([cmatsuoka/asciiquarium](https://github.com/cmatsuoka/asciiquarium)):

```
perl tools/extract.pl path/to/asciiquarium > art.txt
perl tools/generate.pl art.txt > src/art.rs
cargo fmt
```

- `extract.pl` scans every `q{...}` / `q#...#` and multiline `"..."` art literal,
  evals it so Perl produces the exact bytes, and dumps them grouped by the
  enclosing `add_*` sub.
- `generate.pl` turns that dump into byte-exact Rust raw-string constants.
