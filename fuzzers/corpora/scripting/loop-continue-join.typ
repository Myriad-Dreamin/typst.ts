
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test joining with continue.

#let x = for i in range(5) {
  "a"
  if calc.rem(i, 3) == 0 {
    "_"
    continue
  }
  str(i)
}

#test(x, "a_a1a2a_a4")