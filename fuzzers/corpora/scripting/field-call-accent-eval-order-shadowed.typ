
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test shadowing a variable in arguments while calling a method on it.
#{
  let sm = symbol("m", ("method", sym.tilde))
  test(sm.method(let sm = false), sym.tilde(none))
  test(sm, false)
}