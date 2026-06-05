
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import std.calc: pi
#test(pi, calc.pi)