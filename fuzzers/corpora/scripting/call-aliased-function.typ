
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Call function assigned to variable.
#let alias = type
#test(alias(alias), type)