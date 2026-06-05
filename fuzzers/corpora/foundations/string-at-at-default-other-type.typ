
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test("Hello".at(5, default: (a: 10)), (a: 10))