
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: (left: 2.5em))
#set par.line(numbering: "1", numbering-scope: "page")

First line \
Second line
#pagebreak()
Back to first line \
Second line again
#page[
  Once again, first \
  And second
]
Back to first