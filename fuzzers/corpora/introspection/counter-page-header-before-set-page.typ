
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(numbering: "1", number-align: top + center, margin: (top: 20pt))
A
#counter(page).update(4)
#set page(fill: aqua)
B