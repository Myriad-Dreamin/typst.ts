
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
$ mat(a, b; c, d) mat(x; y) $

$ x mat(a; c) + y mat(b; d)
  = mat(a x+b y; c x+d y) $

$ mat(
    -d_0, lambda_0, 0, 0, dots;
    mu_1, -d_1, lambda_1, 0, dots;
    0, mu_2, -d_2, lambda_2, dots;
    dots.v, dots.v, dots.v, dots.v, dots.down;
  )
  mat(p_0; p_1; p_2; dots.v) $