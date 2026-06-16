// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Inline
$a + 5 + cancel(x) + b - cancel(x)$

$c + (a dot.c cancel(b dot.c c))/(cancel(b dot.c c))$