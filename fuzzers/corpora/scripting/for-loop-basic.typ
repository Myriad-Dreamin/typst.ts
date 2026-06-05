
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Empty array.
#for x in () [Nope]

// Dictionary is traversed in insertion order.
// Should output `Name: Typst. Age: 2.`.
#for (k, v) in (Name: "Typst", Age: 2) [
  #k: #v.
]

// Block body.
// Should output `[1st, 2nd, 3rd, 4th]`.
#{
  "["
  for v in (1, 2, 3, 4) {
    if v > 1 [, ]
    [#v]
    if v == 1 [st]
    if v == 2 [nd]
    if v == 3 [rd]
    if v >= 4 [th]
   }
   "]"
}

// Content block body.
// Should output `2345`.
#for v in (1, 2, 3, 4, 5, 6, 7) [#if v >= 2 and v <= 5 { repr(v) }]

// Map captured arguments.
#let f1(..args) = args.pos().map(repr)
#let f2(..args) = args.named().pairs().map(p => repr(p.first()) + ": " + repr(p.last()))
#let f(..args) = (f1(..args) + f2(..args)).join(", ")
#f(1, a: 2)