
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test breaking of equations with numbering.
#set page(height: 5em)
#set math.equation(numbering: "1")
#show math.equation: set block(breakable: true)

$ a &+ b + & c \
  a &+ b   &   && + d \
  a &+ b + & c && + d \
    &      & c && + d \
    &= 0 $