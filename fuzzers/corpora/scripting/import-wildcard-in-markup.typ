
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A wildcard import.
#import "module.typ": *

// It exists now!
#test(d, 3)