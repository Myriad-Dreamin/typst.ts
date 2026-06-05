
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether all footnotes inside another footnote are listed.
#footnote[#footnote[A]#footnote[B]#footnote[C]]