
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test continue.

#let i = 0
#let x = 0

#while x < 8 {
  i += 1
  if calc.rem(i, 3) == 0 {
    continue
  }
  x += i
}

// If continue did not work, this would equal 10.
#test(x, 12)