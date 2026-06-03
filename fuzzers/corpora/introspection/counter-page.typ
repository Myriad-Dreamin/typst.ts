
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 50pt, margin: (bottom: 20pt, rest: 10pt))
#lines(4)
#set page(numbering: "(i)")
#lines(2)
#pagebreak()
#set page(numbering: "1 / 1")
#counter(page).update(1)
#lines(7)