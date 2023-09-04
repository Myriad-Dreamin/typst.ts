
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test forcing a specific style.
$A, italic(A), upright(A), bold(A), bold(upright(A)), \
 serif(A), sans(A), cal(A), frak(A), mono(A), bb(A), \
 italic(diff), upright(diff), \
 bb("hello") + bold(cal("world")), \
 mono("SQRT")(x) wreath mono(123 + 456)$
