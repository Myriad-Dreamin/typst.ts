
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let std = 10
#(std = 7)
#test(std, 7)