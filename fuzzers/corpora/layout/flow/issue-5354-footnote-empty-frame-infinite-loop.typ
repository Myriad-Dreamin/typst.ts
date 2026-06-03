
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether an empty footnote would cause infinite loop
#show footnote.entry: it => {}
#lorem(3) #footnote[A footnote]