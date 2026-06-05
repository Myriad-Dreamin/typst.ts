
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// A renamed module import without items.
#import "module.typ" as other
#test(other.b, 1)
#test(other.item(1, 2), 3)
#test(other.push(2), 3)