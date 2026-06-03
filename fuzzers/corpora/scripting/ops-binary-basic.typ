
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Subtraction.
#test(1-4, 3*-1)
#test(4cm - 2cm, 2cm)
#test(1e+2-1e-2, 99.99)

// Multiplication.
#test(2 * 4, 8)

// Division.
#test(12pt/.4, 30pt)
#test(7 / 2, 3.5)

// Combination.
#test(3-4 * 5 < -10, true)
#test({ let x; x = 1 + 4*5 >= 21 and { x = "a"; x + "b" == "ab" }; x }, true)

// With block.
#test(if true {
  1
} + 2, 3)

// Mathematical identities.
#let nums = (
  1, 3.14,
  decimal("12.45"),
  12pt, 3em, 12pt + 3em,
  45deg,
  90%,
  13% + 10pt, 5% + 1em + 3pt,
  2.3fr,
)

#for v in nums {
  // Test plus and minus.
  test(v + v - v, v)
  test(v - v - v, -v)

  // Test plus/minus and multiplication.
  test(v - v, 0 * v)
  test(v + v, 2 * v)

  // Integer or decimal addition does not give a float.
  if type(v) not in (int, decimal) {
    test(v + v, 2.0 * v)
  }

  if type(v) not in (relative, decimal) and ("pt" not in repr(v) or "em" not in repr(v)) {
    test(v / v, 1.0)
  }
}

// Make sure length, ratio and relative length
// - can all be added to / subtracted from each other,
// - multiplied with integers and floats,
// - divided by integers and floats.
#let dims = (10pt, 1em, 10pt + 1em, 30%, 50% + 3cm, 40% + 2em + 1cm)
#for a in dims {
  for b in dims {
    test(type(a + b), type(a - b))
  }

  for b in (7, 3.14) {
    test(type(a * b), type(a))
    test(type(b * a), type(a))
    test(type(a / b), type(a))
  }
}

// Test division of different numeric types with zero components.
#for a in (0pt, 0em, 0%) {
  for b in (10pt, 10em, 10%) {
    test((2 * b) / b, 2)
    test((a + b * 2) / b, 2)
    test(b / (b * 2 + a), 0.5)
  }
}