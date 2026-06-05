
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 60pt)
#table(
  table.header[@netwok],
  [A],
  [A],
)

#show bibliography: none
#bibliography("/assets/bib/works.bib")