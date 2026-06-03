
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When passing to a function, we lose the italic styling if we wrap the content
// in a non-math function unless it's already nested in some math element (lr,
// attach, etc.)
//
// This is not good, so this test should fail and be updated once it is fixed.
#let id(body) = body
#let bx(body) = box(body, stroke: blue+0.5pt, inset: (x:2pt, y:3pt))
#let eq(body) = math.equation(body)
$
     x y   &&quad     x (y z)   &quad     x y^z  \
  id(x y)  &&quad  id(x (y z))  &quad  id(x y^z) \
  eq(x y)  &&quad  eq(x (y z))  &quad  eq(x y^z) \
  bx(x y)  &&quad  bx(x (y z))  &quad  bx(x y^z) \
$