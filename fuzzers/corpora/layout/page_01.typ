
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Just empty page with styles.
// Should result in one conifer-colored A11 page.
#page("a11", flipped: true, fill: conifer)[]
