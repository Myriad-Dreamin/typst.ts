
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Pagebreak, empty with styles and then pagebreak
// Should result in one auto-sized page and two conifer-colored 2cm wide pages.
#pagebreak()
#set page(width: 2cm, fill: conifer)
#pagebreak()
