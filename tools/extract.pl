#!/usr/bin/env perl
# Extract every q{...} / q#...# art literal from the reference asciiquarium,
# de-escaped to exact bytes by Perl itself, grouped by the enclosing sub.
use strict;
use warnings;

my $src = do { local $/; open my $f, '<', shift or die $!; <$f> };

# Map byte offset -> current sub name, by scanning `sub NAME {` positions.
my @subs;
while ($src =~ /^sub\s+(\w+)/mg) {
    push @subs, [ pos($src), $1 ];
}
sub sub_at {
    my $off = shift;
    my $name = 'main';
    for my $s (@subs) { $name = $s->[1] if $s->[0] <= $off; }
    return $name;
}

my %out;      # sub => [ strings ]
my $i = 0;
my $len = length $src;
while ($i < $len) {
    # find next literal start: q{ or q#
    # multiline double-quoted string literal (used for add_new_monster shapes)
    if (substr($src, $i, 1) eq '"') {
        my $j = $i + 1;
        while ($j < $len) {
            my $c = substr($src, $j, 1);
            if ($c eq "\\") { $j += 2; next; }
            last if $c eq '"';
            $j++;
        }
        my $literal = substr($src, $i, $j - $i + 1);
        if (index($literal, "\n") >= 0) {           # only multiline art
            my $val = eval $literal;
            push @{ $out{ sub_at($i) } }, $val unless $@;
        }
        $i = $j + 1;
        next;
    }
    if (substr($src, $i, 2) eq 'q{' || substr($src, $i, 2) eq 'q#') {
        my $open = substr($src, $i + 1, 1);
        my $close = $open eq '{' ? '}' : '#';
        my $j = $i + 2;
        my $depth = 1;
        while ($j < $len) {
            my $c = substr($src, $j, 1);
            if ($c eq "\\") { $j += 2; next; }   # skip escaped char
            if ($open eq '{' && $c eq '{') { $depth++; }
            elsif ($c eq $close) { $depth--; last if $depth == 0; }
            $j++;
        }
        my $literal = substr($src, $i, $j - $i + 1);   # includes q{ ... }
        my $val = eval $literal;                       # let Perl de-escape
        die "eval failed at offset $i: $@" if $@;
        push @{ $out{ sub_at($i) } }, $val;
        $i = $j + 1;
    } else {
        $i++;
    }
}

# Emit a simple self-delimiting text dump: one record per literal.
# Format:  ===SUB<name>#<index>===\n<bytes>\n===END===\n
for my $name (sort keys %out) {
    my $idx = 0;
    for my $s (@{ $out{$name} }) {
        print "===SUB $name #$idx===\n";
        print $s;
        print "\n===END===\n";
        $idx++;
    }
}
