
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#test((1, 2, 3).zip((1, 2, 3)).map(((_, x)) => x), (1, 2, 3))
