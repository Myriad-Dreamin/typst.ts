
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test braces.
$ x = underbrace(
  1 + 2 + ... + 5,
  underbrace("numbers", x + y)
) $
