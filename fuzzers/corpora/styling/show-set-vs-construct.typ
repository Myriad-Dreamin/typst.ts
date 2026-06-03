
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The constructor property should still work
// when there are recursive show rules.
#show enum: set text(blue)
#enum(numbering: "(a)", [A], enum[B])