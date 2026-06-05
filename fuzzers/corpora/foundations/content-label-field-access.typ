
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether the label is accessible through field syntax.
#show heading: it => {
  assert(str(it.label) == "my-label")
  it
}

= Hello, world! <my-label>