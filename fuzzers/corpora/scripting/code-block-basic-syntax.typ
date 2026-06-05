
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Evaluates to join of none, [My ] and the two loop bodies.
#{
  let parts = ("my fri", "end.")
  [Hello, ]
  for s in parts [#s]
}

// Evaluates to join of the content and strings.
#{
  [How]
  if true {
    " are"
  }
  [ ]
  if false [Nope]
  [you] + "?"
}