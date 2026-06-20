
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Using a non-mutating method, `dict.sym.push()`, in its own argument, but
// indirectly via a mutating method, `sym-sym.pop()`.
#{
  let sp-dict = (sym: symbol("p", ("push", sym.tilde)))
  let array = ("sym", "sym")
  let result = sp-dict
    .at(array.pop())
    .push(
      sp-dict.at(array.pop()).push(none)
    )
  test(result, sym.tilde(sym.tilde(none)))
  test(array, ())
}