
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 8-24 implicit conversion from array to `terms.item` is deprecated
// Hint: 8-24 use `terms.item(term, description)` instead
// Hint: 8-24 this conversion was never documented and is being phased out
#terms(([One], [First]))