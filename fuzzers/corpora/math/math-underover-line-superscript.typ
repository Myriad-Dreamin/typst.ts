
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test effect of lines on superscripts.
$J^b != overline(J)^b != underline(J)^b != underline(overline(J))^b \
 K^3 != overline(K)^3 != underline(K)^3 != underline(overline(K))^3 \
 T^i != overline(T)^i != underline(T)^i != underline(overline(T))^i$