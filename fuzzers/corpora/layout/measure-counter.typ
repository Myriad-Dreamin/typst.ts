
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// When the thing we measure appears multiple times, we measure as if it was
// the first one.
#context {
  let c = counter("c")
  let it = context c.get().first() * h(1pt)
  c.update(42) + it
  metadata(measure(it).width)
}
#context test(query(metadata).first().value, 42pt)