
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Pass closure as argument and return closure.
// Also uses shorthand syntax for a single argument.
#{
  let chain = (f, g) => (x) => f(g(x))
  let f = x => x + 1
  let g = x => 2 * x
  let h = chain(f, g)
  test(h(2), 5)
}