
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: (bottom: 20pt, rest: 0pt))
#let filler = lines(1)

// Test values greater than 32-bits
#set page(numbering: "1/1")
#counter(page).update(100000000001)
#pagebreak()
#pagebreak()