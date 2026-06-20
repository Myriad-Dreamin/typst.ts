
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: the number 100 is too large to be represented with the `arabic.o` numeral system
// Hint: this will become a hard error in the future
#set page(numbering: "①")
#counter(page).update(100)
Hello