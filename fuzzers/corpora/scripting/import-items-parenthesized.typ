
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "module.typ": ()
#import "module.typ": (a)
#import "module.typ": (a, b)
#import "module.typ": (a, b, c, d)

#test(a, none)
#test(b, 1)
#test(c, 2)
#test(d, 3)