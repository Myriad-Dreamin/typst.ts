
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test show-set rule on the same element.
#set figure(supplement: [Default])
#show figure.where(kind: table): set figure(supplement: [Tableau])
#figure(
  table(columns: 2)[A][B][C][D],
  caption: [Four letters],
)