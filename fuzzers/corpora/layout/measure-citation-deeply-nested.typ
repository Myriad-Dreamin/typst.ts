
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested the citation deeply to test that introspector-assisted measurement
// is able to deal with memoization boundaries.
#context {
  let it = box(pad(x: 5pt, grid(stack[@netwok])))
  [#measure(it).width]
  it
}

#show bibliography: none
#bibliography("/assets/bib/works.bib")