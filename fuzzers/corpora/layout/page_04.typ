
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Just page followed by pagebreak.
// Should result in one forest-colored A11 page and one auto-sized page.
#page("a11", flipped: true, fill: forest)[]
#pagebreak()
