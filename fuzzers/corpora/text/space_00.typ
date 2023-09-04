
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Spacing around code constructs.
A#let x = 1;B  #test(x, 1) \
C #let x = 2;D #test(x, 2) \
E#if true [F]G \
H #if true{"I"} J \
K #if true [L] else []M \
#let c = true; N#while c [#(c = false)O] P \
#let c = true; Q #while c { c = false; "R" } S \
T#for _ in (none,) {"U"}V
#let foo = "A" ; \
#foo;B \
#foo; B \
#foo ;B
