
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Basic closure without captures.
#{
  let adder = (x, y) => x + y
  test(adder(2, 3), 5)
}