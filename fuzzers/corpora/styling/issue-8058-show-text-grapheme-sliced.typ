
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The issue is not yet fixed, so this test demonstrates the current undesirable
// behavior.

#set text(font: "New Computer Modern Math")

#let emptyset = sym.emptyset
#let narrowemptysettext = emptyset + "\u{fe00}"
#let narrowemptyset = symbol(narrowemptysettext)

// This should still be narrow, but it's currently not because the symbol is
// split up and the shaper gets both parts separately. This could probably also
// be fixed in the inline layout.
//
// See also https://github.com/typst/typst/issues/8058.
#[
  #show emptyset: it => it
  #narrowemptysettext, $narrowemptyset$
]

// Same here.
#[
  #show "\u{fe00}": it => it
  #narrowemptysettext, $narrowemptyset$
]