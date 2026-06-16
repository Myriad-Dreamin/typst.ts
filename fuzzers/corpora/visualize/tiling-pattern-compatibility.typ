// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto, height: auto, margin: 0pt)

// Warning: 10-17 the name `pattern` is deprecated, use `tiling` instead
// Hint: 10-17 it will be removed in Typst 0.15.0
#let t = pattern(size: (10pt, 10pt), line(stroke: 4pt, start: (0%, 0%), end: (100%, 100%)))
#rect(width: 50pt, height: 50pt, fill: t)
