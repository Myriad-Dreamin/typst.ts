
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#import "module.typ"
#test(module.item(1, 2), 3)