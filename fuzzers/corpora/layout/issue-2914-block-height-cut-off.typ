
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that breaking a block doesn't shrink its height.
#set page(height: 65pt)
#set block(fill: aqua, width: 25pt, height: 25pt, inset: 5pt)

#block[A]
#block[B]