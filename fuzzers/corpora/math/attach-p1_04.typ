
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test associativity and scaling.
$ 1/(V^2^3^4^5),
  1/attach(V, tl: attach(2, tl: attach(3, tl: attach(4, tl: 5)))),
  attach(Omega,
    tl: attach(2, tl: attach(3, tl: attach(4, tl: 5))),
    tr: attach(2, tr: attach(3, tr: attach(4, tr: 5))),
    bl: attach(2, bl: attach(3, bl: attach(4, bl: 5))),
    br: attach(2, br: attach(3, br: attach(4, br: 5))),
  )
$
