
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test how ignorant content affects spacing around alignment points.
#let p = place[]
$
  a + & b + & c & e &    + d \
  a + &     & c &   & #p + d
$