
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test automatic matching.
#set page(width:122pt)
$ (a) + {b/2} + abs(a)/2 + (b) $
$f(x/2) < zeta(c^2 + abs(a + b/2))$
