
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that returning the joined content is not a warning.
#let f() = {
  state("hello").update("world")
  return
}

#test(f(), state("hello").update("world"))