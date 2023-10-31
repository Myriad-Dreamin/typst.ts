
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Citation-format: author-date or author
#set quote(block: true)
#bibliography("/assets/files/works.bib", style: "apa")

#quote(attribution: <tolkien54>)[In a hole in the ground there lived a hobbit.]
