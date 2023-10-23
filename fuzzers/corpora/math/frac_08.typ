
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test precedence.
$ a_1/b_2, 1/f(x), zeta(x)/2, "foo"[|x|]/2 \
  1.2/3.7, 2.3^3.4 \
  🏳️‍🌈[x]/2, f [x]/2, phi [x]/2, 🏳️‍🌈 [x]/2 \
  +[x]/2, 1(x)/2, 2[x]/2 \
  (a)b/2, b(a)[b]/2 \
  n!/2, 5!/2, n !/2, 1/n!, 1/5! $
