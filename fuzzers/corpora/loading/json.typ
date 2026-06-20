
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reading JSON data.
#let data = json("/assets/data/zoo.json")
#test(data.len(), 3)
#test(data.at(0).name, "Debby")
#test(data.at(2).weight, 150)

// Test reading through path type.
#let data-from-path = json(path("/assets/data/zoo.json"))
#test(data-from-path, data)