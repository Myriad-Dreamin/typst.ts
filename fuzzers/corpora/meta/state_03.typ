
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Make sure that a warning is produced if the layout fails to converge.
// Warning: layout did not converge within 5 attempts
// Hint: check if any states or queries are updating themselves
#let s = state("s", 1)
#locate(loc => s.update(s.final(loc) + 1))
#s.display()
