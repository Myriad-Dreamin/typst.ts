
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test setting the binding explicitly.
#set page(margin: (inside: 30pt))
#rect(width: 100%)[Bound]
#pagebreak()
#rect(width: 100%)[Left]
