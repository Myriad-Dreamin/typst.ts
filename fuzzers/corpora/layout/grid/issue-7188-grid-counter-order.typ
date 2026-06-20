
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 1cm)

#let word-numbering(body) = {
  let num = counter("_linenumbered")
  let word-label = <_word>
  show word-label: _ => {
    num.step()
    box(width: 0pt, super(numbering("1", num.get().first())))
  }
  show regex("\\w+\\.?"): it => it + [#metadata(none)#word-label]
  body
}

#grid(
  columns: 1,
  word-numbering(lorem(8))
)