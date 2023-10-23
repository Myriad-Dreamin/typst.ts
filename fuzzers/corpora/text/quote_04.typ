
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Citation-format: label or numeric
#set quote(block: true)
#bibliography("/assets/files/works.bib", style: "ieee")

#quote(attribution: <tolkien54>)[In a hole in the ground there lived a hobbit.]
