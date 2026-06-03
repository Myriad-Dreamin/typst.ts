
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test it again with a different element.
#set heading(numbering: "(I)")
#show heading: set text(size: 11pt, weight: "regular")
#show heading: it => it.numbering
= Heading