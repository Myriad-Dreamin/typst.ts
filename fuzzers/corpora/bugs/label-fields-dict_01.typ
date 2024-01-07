
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test whether the label is accessible through the field method
#show heading: it => {
  assert(str(it.label) == "my_label")
  it
}

= Hello, world! <my_label>
