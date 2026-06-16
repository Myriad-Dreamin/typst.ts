// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test script attachments positioning if the base is an extended shape (or a
// sequence of extended shapes).
$lr(size: #130%, [x])_0^1, [x]_0^1, \]_0^1, x_0^1, A_0^1$ \
$n^2, (n + 1)^2, sum_0^1, integral_0^1$