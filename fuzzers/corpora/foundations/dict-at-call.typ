
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test calling a function in a dictionary via `.at()`.
#let dict = (func: x => x + 1)
#test(dict.at("func")(0), 1)