
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that discarding markup is not a warning.
#let f() = [
  hello
  #return [nope]
]

#test(f(), [nope])