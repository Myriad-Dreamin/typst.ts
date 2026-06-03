
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test return with value.
#let f(x) = {
  return x + 1
}

#test(f(1), 2)