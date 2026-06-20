
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: document did not converge within five attempts
// Hint: see 5 additional warnings for more details
// Hint: see https://typst.app/help/convergence for help
#import "switch.typ": switch
#show strong: none

= Real <real>

#context {
  // Warning: 15-29 number of heading elements did not stabilize
  // Hint: 15-29 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 1\n- run 3: 2\n- run 4: 3\n- run 5: 4\n- final: 5
  let elems = query(heading)
  let count = elems.len()
  count * [= Fake <fake>]
}

#context {
  // This one converges.
  _ = query(<real>)
}

// Test alternative warning messages.
#context {
  // Warning: 7-37 number of matching heading elements did not stabilize
  // Hint: 7-37 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 1\n- run 3: 2\n- run 4: 3\n- run 5: 4\n- final: 5
  _ = query(heading.where(level: 1))

  // Warning: 7-20 number of elements labelled `<fake>` did not stabilize
  // Hint: 7-20 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 0\n- run 3: 1\n- run 4: 2\n- run 5: 3\n- final: 4
  _ = query(<fake>)

  // Warning: 7-48 number of elements matching `selector.or(<fake>, heading.where(level: 1))` did not stabilize
  // Hint: 7-48 the following numbers of elements were observed:\n- run 1: 0\n- run 2: 1\n- run 3: 2\n- run 4: 3\n- run 5: 4\n- final: 5
  _ = query(heading.where(level: 1).or(<fake>))
}

// This one has no hint since the number of matching elements is the same and
// it's difficult to provide a good hint for the concrete elements.
#switch(n => if n == 4 [*A*] else [*B*])
#context {
  // Warning: 7-20 query for strong elements did not stabilize
  _ = query(strong)
}