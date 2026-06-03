
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Citation-format: note
#set text(8pt)
#set quote(block: true)
#quote(attribution: <tolkien54>)[In a hole in the ground there lived a hobbit.]

#show bibliography: none
#bibliography("/assets/bib/works.bib", style: "chicago-shortened-notes")