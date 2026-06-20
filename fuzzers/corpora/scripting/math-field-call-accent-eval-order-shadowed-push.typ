
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Math doesn't support mutable methods and always evaluates arguments second.
#{
  let sp = symbol("p", ("push", sym.tilde))
  test($sp.push(#let sp = false;)$, $#sym.tilde(none)$)
  test(sp, false)
}