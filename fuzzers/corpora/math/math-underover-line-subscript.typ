
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test effect of lines on subscripts.
$A_2 != overline(A)_2 != underline(A)_2 != underline(overline(A))_2 \
 V_y != overline(V)_y != underline(V)_y != underline(overline(V))_y \
 W_l != overline(W)_l != underline(W)_l != underline(overline(W))_l$