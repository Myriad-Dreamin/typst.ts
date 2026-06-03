
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Value of while loops.

#test(while false {}, none)

#let i = 0
#test(type(while i < 1 [#(i += 1)]), content)