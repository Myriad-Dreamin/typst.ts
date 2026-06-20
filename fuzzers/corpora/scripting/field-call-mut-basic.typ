
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Mutating methods mutate a variable.
#let numbers = (1, 2, 3)
#test(numbers.remove(1), 2)
#test(numbers, (1, 3))