
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// This also wasn't allowed.
#let ((a, b)) = (1, 2)
#test(a, 1)
#test(b, 2)