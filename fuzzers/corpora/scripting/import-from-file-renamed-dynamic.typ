
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "mod" + "ule.typ" as mod
#test(mod.b, 1)