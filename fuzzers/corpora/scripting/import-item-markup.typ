
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// An item import.
#import "module.typ": item
#test(item(1, 2), 3)