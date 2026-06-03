
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Measure a counter. Tests that the introspector-assisted location assignment
// is able to take `here()` from the context into account to find the closest
// matching element instead of any single one. Crucially, we need to reuse
// the same `context c.display()` to get the same span, hence `it`.
#let f(it) = context [
  Is #measure(it).width wide: #it \
]

#let c = counter("c")
#let it = context c.display()

#c.update(10000)
#f(it)
#c.update(100)
#f(it)
#c.update(1)
#f(it)