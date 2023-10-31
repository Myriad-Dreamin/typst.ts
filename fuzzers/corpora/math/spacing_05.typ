
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test weak spacing
$integral f(x) dif x$,
// Not weak
$integral f(x) thin dif x$,
// Both are weak, collide
$integral f(x) #h(0.166em, weak: true)dif x$
