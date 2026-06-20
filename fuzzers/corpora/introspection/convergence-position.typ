
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 1 additional warning for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch

// Hint: 29-38 heading was created here
#switch(n => v(n * 10pt) + [= Heading])

// Warning: 10-36 heading position did not stabilize
// Hint: 10-36 the following positions were observed:\n- run 1: page 1 at (0pt, 0pt)\n- run 2: page 1 at (10pt, 20pt)\n- run 3: page 1 at (10pt, 30pt)\n- run 4: page 1 at (10pt, 40pt)\n- run 5: page 1 at (10pt, 50pt)\n- final: page 1 at (10pt, 60pt)
#context locate(heading).position().y