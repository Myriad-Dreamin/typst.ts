
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tests that the determination of the matching numbering is comprehensive for
// all supported elements.

// This should be overridden by the element's numbering.
#set heading(numbering: "(i)")
#set math.equation(block: true)

#let funcs = (heading, figure, math.equation, footnote)
#show selector.or(..funcs): it => counter(it.func()).display()
#for f in funcs {
  block(f(numbering: "a)")[])
}