
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test hard and weak pagebreak followed by page with body.
// Should result in three navy-colored pages.
#set page(fill: navy)
#set text(fill: white)
First
#pagebreak()
#page[Second]
#pagebreak(weak: true)
#page[Third]
