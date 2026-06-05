
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test custom page fields being applied on the last page
// if the document has custom fields.
#set document(author: "")
#set page(fill: gray)
text
#pagebreak()