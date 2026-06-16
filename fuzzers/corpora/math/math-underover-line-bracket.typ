// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test lines and brackets.
$ x = overbracket(
  overline(underline(x + y)),
  1 + 2 + ... + 5,
) $