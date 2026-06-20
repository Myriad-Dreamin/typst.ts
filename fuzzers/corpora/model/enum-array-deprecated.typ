
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 7-19 implicit conversion from array to `enum.item` is deprecated
// Hint: 7-19 use `enum.item(number)[body]` instead
// Hint: 7-19 this conversion was never documented and is being phased out
#enum((1, [First]))