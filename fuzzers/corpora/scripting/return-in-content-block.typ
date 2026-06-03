
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test value return from content.
#let x = 3
#let f() = [
  Hello 😀
  #return "nope"
  World
]

#test(f(), "nope")