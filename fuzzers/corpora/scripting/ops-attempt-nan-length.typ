
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that trying to produce a NaN scalar (such as in lengths) does not crash.
#let infpt = float("inf") * 1pt
#test(infpt - infpt, 0pt)
#test(infpt + (-infpt), 0pt)
// TODO: this result is surprising
#test(infpt / float("inf"), 0pt)