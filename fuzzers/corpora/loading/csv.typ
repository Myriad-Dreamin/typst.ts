
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reading CSV data.
#set page(width: auto)
#let data = csv("/assets/data/zoo.csv")
#let cells = data.at(0).map(strong) + data.slice(1).flatten()
#table(columns: data.at(0).len(), ..cells)