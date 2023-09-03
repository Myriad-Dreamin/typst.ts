
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test lines and brackets.
$ x = overbracket(
  overline(underline(x + y)),
  1 + 2 + ... + 5,
) $
