
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test calling a function in an argument via `.at()`.
#let args = arguments(x => x + 1, func: x => x + 2)
#test(args.at(0)(0), 1)
#test(args.at("func")(0), 2)