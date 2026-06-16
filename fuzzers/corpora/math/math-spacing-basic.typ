// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spacing cases.
$ä, +, c, (, )$ \
$=), (+), {times}$ \
$⟧<⟦, abs(-), [=$ \
$a=b, a==b$ \
$-a, +a$ \
$a not b$ \
$a+b, a*b$ \
$sum x, sum(x)$ \
$sum product x$ \
$f(x), zeta(x), "frac"(x)$ \
$a+dots.c+b$
$f(x) sin(y)$