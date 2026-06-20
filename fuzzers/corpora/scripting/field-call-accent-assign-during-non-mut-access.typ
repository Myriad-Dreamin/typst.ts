
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Using a non-mutating method, `dict.sym.push()`, in an assignment, but
// indirectly via a mutating method, `sym-sym.pop()`.
#{
  let sp-dict = (sym: symbol("p", ("push", sym.tilde)))
  let array = ("sym", "sym")
  sp-dict.at(array.pop()) = sp-dict.at(array.pop()).push(none)
  test(array, ())
}