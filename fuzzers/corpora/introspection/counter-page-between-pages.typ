
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The update happens conceptually between the pages.
#set page(numbering: "1", margin: (bottom: 20pt))
A
#pagebreak()
#counter(page).update(5)
#set page(number-align: top + center, margin: (top: 20pt, bottom: 10pt))
B