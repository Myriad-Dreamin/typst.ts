
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set footnote(numbering: "①")
#counter(footnote).update(100)
// Warning: 2-12 the number 101 is too large to be represented with the `arabic.o` numeral system
// Hint: 2-12 this will become a hard error in the future
#footnote[]