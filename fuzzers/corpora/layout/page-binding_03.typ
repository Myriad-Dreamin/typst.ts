
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test setting the binding implicitly.
#set page(margin: (inside: 30pt))
#set text(lang: "he")
#rect(width: 100%)[Bound]
#pagebreak()
#rect(width: 100%)[Right]
