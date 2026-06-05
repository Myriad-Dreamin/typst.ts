
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A renamed item import.
#import "module.typ": item as something
#test(something(1, 2), 3)