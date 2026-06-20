
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 2 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help
#let c = counter("hi")

// Warning: 16-27 value of `counter("hi")` did not converge
// Hint: 16-27 the following values were observed:\n- run 1: 0\n- run 2: 0, 1\n- run 3: 0, 1, 3\n- run 4: 0, 1, 3, 7\n- run 5: 0, 1, 3, 7, 15\n- final: 0, 1, 3, 7, 15, 31
#context { _ = c.at(<end>) }

#context c.update({
  // Warning: 11-20 value of `counter("hi")` did not converge
  // Hint: 11-20 the following values were observed:\n- run 1: 0\n- run 2: 0, 1\n- run 3: 0, 1, 3\n- run 4: 0, 1, 3, 7\n- run 5: 0, 1, 3, 7, 15\n- final: 0, 1, 3, 7, 15, 31
  let v = c.final()
  v + (1 + v.last() * 2,)
})

#metadata(none) <end>