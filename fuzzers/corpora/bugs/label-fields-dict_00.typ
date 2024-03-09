
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test whether the label is accessible through the has method
#show heading: it => {
  assert(it.has("label"))
  it
}

= Hello, world! <my_label>
