
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// String is joined with trailing none, evaluates to string.
#test({
  type("")
  none
}, str)