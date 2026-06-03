
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test condition evaluation.
#if 1 < 2 [
  One.
]

#if true == false [
  {Bad}, but we {dont-care}!
]