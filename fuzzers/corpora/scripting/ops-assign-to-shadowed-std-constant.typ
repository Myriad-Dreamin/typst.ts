
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Works if we define rect beforehand
// (since then it doesn't resolve to the standard library version anymore).
#let rect = ""
#(rect = "hi")