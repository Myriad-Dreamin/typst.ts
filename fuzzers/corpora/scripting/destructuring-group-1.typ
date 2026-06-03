
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This wasn't allowed.
#let ((x)) = 1
#test(x, 1)