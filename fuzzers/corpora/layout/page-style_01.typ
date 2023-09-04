
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Empty with styles and then pagebreak
// Should result in two forest-colored pages.
#set page(fill: forest)
#pagebreak()
