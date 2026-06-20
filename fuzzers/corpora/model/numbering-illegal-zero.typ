
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(numbering("१", 0), "०")
// Warning: 2-19 the numeral system `korean.syllable` cannot represent zero
// Hint: 2-19 this will become a hard error in the future
#numbering("가", 0)