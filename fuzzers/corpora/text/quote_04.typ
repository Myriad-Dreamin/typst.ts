
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Citation-format: label or numeric
#set text(8pt)
#set quote(block: true)
#quote(attribution: <tolkien54>)[In a hole in the ground there lived a hobbit.]

#set text(0pt)
#bibliography("/assets/files/works.bib", style: "ieee")
