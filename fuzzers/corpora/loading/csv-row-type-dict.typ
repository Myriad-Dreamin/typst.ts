
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reading CSV data with dictionary rows enabled.
#let data = csv("/assets/data/zoo.csv", row-type: dictionary)
#test(data.len(), 3)
#test(data.at(0).Name, "Debby")
#test(data.at(2).Weight, "150kg")
#test(data.at(1).Species, "Tiger")