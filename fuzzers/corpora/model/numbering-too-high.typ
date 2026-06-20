
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 2-20 the number 51 is too large to be represented with the `arabic.o` numeral system
// Hint: 2-20 this will become a hard error in the future
#numbering("①", 51)