
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test whether the label is accessible through the fields method
#show heading: it => {
  assert("label" in it.fields())
  assert(str(it.fields().label) == "my_label")
  it
}

= Hello, world! <my_label>
