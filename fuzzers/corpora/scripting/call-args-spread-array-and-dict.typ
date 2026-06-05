
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test spreading array and dictionary.
#{
  let more = (3, -3, 6, 10)
  test(calc.min(1, 2, ..more), -3)
  test(calc.max(..more, 9), 10)
  test(calc.max(..more, 11), 11)
}

#{
  let more = (c: 3, d: 4)
  let tostr(..args) = repr(args)
  test(tostr(a: 1, ..more, b: 2), "arguments(a: 1, c: 3, d: 4, b: 2)")
}