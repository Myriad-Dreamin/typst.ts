
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Tests that when a citation footnote is pushed to next page, things still
// work as expected.
#set page(height: 60pt)
A

#footnote[@netwok]
#show bibliography: none
#bibliography("/assets/bib/works.bib")