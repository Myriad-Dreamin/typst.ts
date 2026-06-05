
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that an invisible tag does not prevent hyphenation.
#set page(width: 50pt)
#set text(hyphenate: true)
#show "Tree": emph
#show emph: set text(red)
#show emph: it => it + metadata(none)
Treebeard