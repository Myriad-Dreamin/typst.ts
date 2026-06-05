
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that discarding joined content is a warning.
#let f() = {
  [Hello, World!]
  // Warning: 3-16 this return unconditionally discards the content before it
  // Hint: 3-16 try omitting the `return` to automatically join all values
  return "nope"
}

#test(f(), "nope")