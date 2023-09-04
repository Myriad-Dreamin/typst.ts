
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test space collapsing before footnote.
A#footnote[A] \
A #footnote[A]
