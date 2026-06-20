
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(arguments().len(), 0)
#test(arguments("hello").len(), 1)
#test(arguments(a: "world").len(), 1)
#test(arguments(a: "hey", 14).len(), 2)
#test(arguments(0, 1, a: 2, 3).len(), 4)