
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set block(spacing: 3pt)
#let c = counter("c")
#let foo() = context {
  c.step()
  c.display("1")
  str(c.get().first())
}

#foo()
#block(foo())
#foo()
#foo()
#block(foo())
#block(foo())
#foo()