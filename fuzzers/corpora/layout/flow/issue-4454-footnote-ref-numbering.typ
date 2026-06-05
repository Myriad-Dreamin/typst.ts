
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that footnote references are numbered correctly.
A #footnote(numbering: "*")[B]<fn>, C @fn, D @fn, E @fn.