
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Mixing renamed and not renamed items.
#import "module.typ": fn, b as val, item as other
#test(val, 1)
#test(other(1, 2), 3)