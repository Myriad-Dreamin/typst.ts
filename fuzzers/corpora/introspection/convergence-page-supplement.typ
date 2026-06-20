
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 2 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch

#set page(numbering: "1", margin: (bottom: 20pt))
#show: doc => switch(n => {
  set page(supplement: "Pagus", numbering: "I") if n == 4
  doc
})

// Hint: 1-8 heading was created here
// Hint: 1-8 heading was created here
= Hello <hello>

// Warning: 2-28 supplement of the page on which the heading is located did not stabilize
// Warning: 2-28 numbering of the page on which the heading is located did not stabilize
// Hint: 2-28 the following supplements were observed:\n- run 1: `[]`\n- run 2: `[page]`\n- run 3: `[page]`\n- run 4: `[page]`\n- run 5: `[Pagus]`\n- final: `[page]`
// Hint: 2-28 the following numberings were observed:\n- run 1: `none`\n- run 2: `"1"`\n- run 3: `"1"`\n- run 4: `"1"`\n- run 5: `"I"`\n- final: `"1"`
#ref(<hello>, form: "page")