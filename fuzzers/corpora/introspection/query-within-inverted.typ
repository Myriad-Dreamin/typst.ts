
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test a case where the ancestor is fully contained in one of the children.
#let m(l) = [#metadata(none)#l]
#strong({
  m(<a>)
  m(<b>)
  m(<c>)
})
#context test(query(selector.within(strong, <b>)), ())