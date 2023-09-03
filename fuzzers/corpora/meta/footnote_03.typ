
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Currently, numbers a bit out of order if a nested footnote ends up in the
// same frame as another one. :(
#footnote[A, #footnote[B]], #footnote[C]
