
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test weak pagebreak before bibliography.
#pagebreak(weak: true)
#bibliography("/assets/bib/works.bib")