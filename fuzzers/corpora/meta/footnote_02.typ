
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test nested footnotes.
First \
Second #footnote[A, #footnote[B, #footnote[C]]] \
Third #footnote[D, #footnote[E]] \
Fourth
