
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 2 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch
#switch(n => if n == 4 [*A* <a>] else if n == 5 [_A_ <a>])

// Warning: 2-17 query for a unique element labelled `<a>` did not stabilize
// Warning: 2-17 query for the first element matching `location(..)` did not stabilize
// Hint: 2-17 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 0\n- run 3: 0\n- run 4: 0\n- run 5: 1\n- final: 0
#link(<a>)[Link]