
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The `first` and `last` methods.
#test((1,).first(), 1)
#test((2,).last(), 2)
#test((1, 2, 3).first(), 1)
#test((1, 2, 3).last(), 3)
#test((1, 2).first(default: 99), 1)
#test(().first(default: 99), 99)
#test((1, 2).last(default: 99), 2)
#test(().last(default: 99), 99)