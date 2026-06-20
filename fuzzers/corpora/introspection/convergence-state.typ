
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 2 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help

#let hi = state("hi", 0)
#context hi.update(hi.get() + 1)
#context hi.update(hi.get() + 2)
#context hi.update(hi.get() + 3)
#context hi.update(hi.get() + 4)
#context hi.update(hi.get() + 5)

// Warning: 20-28 value of `state("hi")` did not converge
// Hint: 20-28 the following values were observed:\n- run 1: `0`\n- run 2: `5`\n- run 3: `9`\n- run 4: `12`\n- run 5: `14`\n- final: `15`
// Hint: 20-28 see https://typst.app/help/state-convergence for help
#context hi.update(hi.get() + 6)

#let s = state("s", 1)

// Warning: 19-28 value of `state("s")` did not converge
// Hint: 19-28 the following values were observed:\n- run 1: `1`\n- run 2: `2`\n- run 3: `3`\n- run 4: `4`\n- run 5: `5`\n- final: `6`
// Hint: 19-28 see https://typst.app/help/state-convergence for help
#context s.update(s.final() + 1)