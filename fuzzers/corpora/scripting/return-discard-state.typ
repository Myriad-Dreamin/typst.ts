
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that discarding a joined content with state is special warning

#let f() = {
  state("hello").update("world")

  // Warning: 3-19 this return unconditionally discards the content before it
  // Hint: 3-19 try omitting the `return` to automatically join all values
  // Hint: 3-19 state/counter updates are content that must end up in the document to have an effect
  return [ Hello ]
}

#test(f(), [ Hello ])