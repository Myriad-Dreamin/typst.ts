
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let f() = {
  [Hello, World!]
  {
    // Warning: 5-18 this return unconditionally discards the content before it
    // Hint: 5-18 try omitting the `return` to automatically join all values
    return "nope"
  }
}

#test(f(), "nope")