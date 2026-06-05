
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Mutable method with capture in argument.
#let x = "b"
#let f() = {
  let a = (b: 5)
  a.at(x) = 10
  a
}
#f()