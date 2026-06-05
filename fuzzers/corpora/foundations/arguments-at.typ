
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let args = arguments(0, 1, a: 2, 3)
#test(args.at(0), 0)
#test(args.at(1), 1)
#test(args.at(2), 3)
#test(args.at("a"), 2)