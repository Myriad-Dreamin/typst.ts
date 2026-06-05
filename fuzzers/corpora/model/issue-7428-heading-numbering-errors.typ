
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that the error from the first layout iteration is silenced.
#set heading(numbering: (n, ..nums) => {
  assert(n > 0)
  [#n]
})

= A