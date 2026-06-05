
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Footer should be affected by default.
#set page(numbering: "1 / 1", margin: (bottom: 20pt))
#counter(page).update(5)