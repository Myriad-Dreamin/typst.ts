
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test math method call for an accent symbol.
#test($arrow.l.r(x)$, $#math.arrow.l.r[\x]$)