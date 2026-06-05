
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Multiple nested scopes.
#{
  let a = "a1"
  {
    let a = "a2"
    {
      test(a, "a2")
      let a = "a3"
      test(a, "a3")
    }
    test(a, "a2")
  }
  test(a, "a1")
}