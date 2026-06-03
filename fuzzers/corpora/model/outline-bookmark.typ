
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that `bookmarked` option doesn't affect the outline
#set heading(numbering: "(I)", bookmarked: false)
#set outline.entry(fill: none)
#show heading: none
#outline()

= A