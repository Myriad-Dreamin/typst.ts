
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 60pt)
#lorem(4)

#footnote[@netwok]
#show bibliography: none
#bibliography("/assets/files/works.bib")
