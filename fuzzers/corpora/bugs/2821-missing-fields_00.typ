
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#show figure.caption: it => {
  assert(it.has("supplement"))
  assert(it.supplement == none)
}
#figure([], caption: [], supplement: none)
