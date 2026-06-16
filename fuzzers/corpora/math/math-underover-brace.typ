// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test braces.
$ x = underbrace(
  1 + 2 + ... + 5,
  underbrace("numbers", x + y)
) $