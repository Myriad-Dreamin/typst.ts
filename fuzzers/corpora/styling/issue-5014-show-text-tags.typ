
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  let c = counter("c")
  show "b": context c.get().first()
  [a]
  c.step()
  [bc]
}