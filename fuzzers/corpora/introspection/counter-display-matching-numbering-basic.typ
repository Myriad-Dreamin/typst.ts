
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that `counter(heading).display()` just works: It takes care of
// using the correct location and numbering.
#show heading: it => block(counter(heading).display() + [ ] + it.body)
#heading(numbering: "1.")[One]
#heading(numbering: "A.")[Two]