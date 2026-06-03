
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(type(1), int)
#test(type(ltr), direction)
#test(type(10 / 3), float)