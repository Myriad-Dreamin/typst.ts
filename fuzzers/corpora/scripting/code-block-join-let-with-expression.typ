
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Evaluated to int.
#test({
  let x = 1
  let y = 2
  x + y
}, 3)