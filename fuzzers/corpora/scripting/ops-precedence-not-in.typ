
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Not in handles precedence.
#test(-1 not in (1, 2, 3), true)