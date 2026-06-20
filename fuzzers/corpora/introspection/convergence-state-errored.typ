
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 1 additional warning for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch
#let s = state("s")
#switch(n => s.update(if n == 5 { _ => panic() } else { "ok" }))

// Warning: 16-23 value of `state("s")` did not converge
// Hint: 16-23 the following values were observed:\n- run 1: `none`\n- run 2: `"ok"`\n- run 3: `"ok"`\n- run 4: `"ok"`\n- run 5: `"ok"`\n- final: (errored)
// Hint: 16-23 see https://typst.app/help/state-convergence for help
#context { _ = s.get() }