
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test in case distinction.
$ f := cases(
  1 + 2 &"iff" &x,
  3     &"if"  &y,
) $
