
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A module import without items.
#import "module.typ"
#test(module.b, 1)
#test(module.item(1, 2), 3)
#test(module.push(2), 3)