
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test brackets.
$ underbracket([1, 2/3], "relevant stuff")
          arrow.l.r.double.long
  overbracket([4/5,6], "irrelevant stuff") $
