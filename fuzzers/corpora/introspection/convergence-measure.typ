
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 1 additional warning for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch
#switch(n => {
  // Hint: 15-22 the closest match for this element did not stabilize
  let body = [= Hello]

  // Warning: 7-20 a measured element did not stabilize
  // Hint: 7-20 measurement tries to resolve introspections by finding the closest matching elements in the real document
  _ = measure(body)
  if n == 4 {
    body
  }
})