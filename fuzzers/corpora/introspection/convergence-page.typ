
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 2 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch

// No "heading was created here" hint because the heading does not exist anymore
// in the end.
#switch(n => if n == 4 { pagebreak() + [= Heading] })

// Warning: 10-25 query for a unique heading element did not stabilize
// Hint: 10-25 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 0\n- run 3: 0\n- run 4: 0\n- run 5: 1\n- final: 0
// Warning: 10-32 page number of the element did not stabilize
// Hint: 10-32 the following page numbers were observed:\n- run 1: page 1\n- run 2: page 1\n- run 3: page 1\n- run 4: page 1\n- run 5: page 2\n- final: page 1
#context locate(heading).page()