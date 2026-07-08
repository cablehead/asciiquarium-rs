#!/usr/bin/env perl
# Turn the extracted art.txt into a byte-exact Rust data module (src/art.rs).
use strict;
use warnings;

my %group = (
    add_environment  => 'WATER',
    add_castle       => 'CASTLE',
    add_new_fish     => 'NEW_FISH',
    add_old_fish     => 'OLD_FISH',
    add_shark        => 'SHARK',
    add_ship         => 'SHIP',
    add_whale        => 'WHALE',
    add_new_monster  => 'NEW_MONSTER',
    add_old_monster  => 'OLD_MONSTER',
    add_big_fish_1   => 'BIG_FISH_1',
    add_big_fish_2   => 'BIG_FISH_2',
    add_splat        => 'SPLAT',
);
# Emit order (readability).
my @order = qw(add_environment add_castle add_new_fish add_old_fish add_shark
    add_ship add_whale add_new_monster add_old_monster add_big_fish_1
    add_big_fish_2 add_splat);

my $txt = do { local $/; <> };
my %items;   # sub => [ strings in index order ]
while ($txt =~ /===SUB (\S+) #(\d+)===\n(.*?)\n===END===\n/gs) {
    $items{$1}[$2] = $3;
}

print <<'HEADER';
// @generated from the reference Perl by tools/generate.pl -- do not edit.
//
// Byte-exact ASCII art, de-escaped by Perl from cmatsuoka/asciiquarium. Every
// block is a raw string so backslashes stay literal. In these sprites '?' is
// the transparency character (Term::Animation's default) and, where auto_trans
// is set, ' ' is transparent too. See sprites.rs for how they are grouped into
// creatures.

HEADER

for my $sub (@order) {
    my $name = $group{$sub};
    my @arr  = @{ $items{$sub} };
    my $n    = scalar @arr;
    print "pub const $name: [&str; $n] = [\n";
    for my $s (@arr) {
        print "    r#\"$s\"#,\n";
    }
    print "];\n\n";
}

# Bubble frames are a plain char list in the Perl, not a q{} literal.
print <<'TAIL';
/// Bubble growth frames, smallest to largest.
pub const BUBBLE: [&str; 5] = [".", "o", "O", "O", "O"];
TAIL
