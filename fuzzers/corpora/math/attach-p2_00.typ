
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test high subscript and superscript.
$ sqrt(a_(1/2)^zeta), sqrt(a_alpha^(1/2)), sqrt(a_(1/2)^(3/4)) \
  sqrt(attach(a, tl: 1/2, bl: 3/4)),
  sqrt(attach(a, tl: 1/2, bl: 3/4, tr: 1/2, br: 3/4)) $
