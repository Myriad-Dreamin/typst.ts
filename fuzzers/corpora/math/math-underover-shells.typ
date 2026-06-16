// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test tortoise shell brackets.
$ undershell(
  1 + overshell(2 + ..., x + y),
  "all stuff"
) $