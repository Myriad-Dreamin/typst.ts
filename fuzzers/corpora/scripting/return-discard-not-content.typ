
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that non-content joined value is not a warning.
#let f() = {
  (33,)
  return (66,)
}

#test(f(), (66, ))