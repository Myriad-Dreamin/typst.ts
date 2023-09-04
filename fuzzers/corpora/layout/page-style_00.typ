
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Empty with styles
// Should result in one conifer-colored A11 page.
#set page("a11", flipped: true, fill: conifer)
