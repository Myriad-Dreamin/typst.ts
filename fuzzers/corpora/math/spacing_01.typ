
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test ignored vs non-ignored spaces.
$f (x), f(x)$ \
$[a|b], [a | b]$ \
$a"is"b, a "is" b$
