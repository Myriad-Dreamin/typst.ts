
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test doing things with arguments.
#{
  let save(..args) = {
    test(type(args), arguments)
    test(repr(args), "arguments(three: true, 1, 2)")
  }

  save(1, 2, three: true)
}