
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that scoping works as expected.
#{
  if true {
    set text(blue)
    [Blue ]
  }
  [Not blue]
}