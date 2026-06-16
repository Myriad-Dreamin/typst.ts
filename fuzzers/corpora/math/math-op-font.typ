// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with different font.
#let colim = math.op(
  text(font: "IBM Plex Sans", weight: "regular", size: 0.8em)[colim],
  limits: true,
)
$ colim_(x -> 0) inline(colim_(x -> 0)) $