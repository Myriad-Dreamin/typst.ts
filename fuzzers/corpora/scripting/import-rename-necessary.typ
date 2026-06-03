
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "module.typ" as module: a
#test(module.a, a)