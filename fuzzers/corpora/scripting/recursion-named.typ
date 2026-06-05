
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test with named function.
#let fib(n) = {
  if n <= 2 {
    1
  } else {
    fib(n - 1) + fib(n - 2)
  }
}

#test(fib(10), 55)