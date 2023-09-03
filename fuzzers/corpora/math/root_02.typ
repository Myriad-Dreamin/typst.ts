
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test precomposed vs constructed roots.
// 3 and 4 are precomposed.
$sqrt(x)$
$root(2, x)$
$root(3, x)$
$root(4, x)$
$root(5, x)$
