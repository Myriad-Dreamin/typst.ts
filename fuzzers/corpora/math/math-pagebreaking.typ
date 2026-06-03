
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test breaking of equations at page boundaries.
#set page(height: 5em)
#show math.equation: set block(breakable: true)

$ a &+ b + & c \
  a &+ b   &   && + d \
  a &+ b + & c && + d \
    &      & c && + d \
    &= 0 $