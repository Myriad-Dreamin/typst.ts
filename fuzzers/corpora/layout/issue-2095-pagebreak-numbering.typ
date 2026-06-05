
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The empty page 2 should not have a page number
#set page(numbering: none)
This and next page should not be numbered

#pagebreak(weak: true, to: "odd")

#set page(numbering: "1")
#counter(page).update(1)

This page should