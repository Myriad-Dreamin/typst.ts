
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Inline citation
#set text(8pt)
#quote(attribution: <tolkien54>)[In a hole in the ground there lived a hobbit.]

#set text(0pt)
#bibliography("/assets/files/works.bib")
