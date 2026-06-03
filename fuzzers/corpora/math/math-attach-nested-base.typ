
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test attachments when the base has attachments.
$ attach(a^b, b: c) quad
  attach(attach(attach(attach(attach(attach(sum, tl: 1), t: 2), tr: 3), br: 4), b: 5), bl: 6) $

#let a0 = math.attach(math.alpha, b: [0])
#let a1 = $alpha^1$
#let a2 = $attach(a1, bl: 3)$

$ a0 + a1 + a0_2 \
  a1_2 + a0^2 + a1^2 \
  a2 + a2_2 + a2^2 $