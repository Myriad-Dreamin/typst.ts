// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test nested lr calls.
#let body1 = math.lr($|$, size: 4em)
#let body2 = $lr(|, size: #4em)$

$lr(|, size: #2em)$
$lr(lr(|, size: #4em), size: #50%)$
$lr(body1, size: #50%)$
$lr(body2, size: #50%)$