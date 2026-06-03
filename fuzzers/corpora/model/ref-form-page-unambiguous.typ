
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that page reference is not ambiguous.
#set page(numbering: "1")

= Introduction <arrgh>

#ref(<arrgh>, form: "page")
#bibliography("/assets/bib/works.bib")