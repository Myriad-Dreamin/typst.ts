
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(numbering: "1", margin: (bottom: 20pt))
A
#pagebreak()
#counter(page).update(5)
#set page(fill: aqua)
B