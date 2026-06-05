
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test setting font.
#show math.equation: set text(weight: "regular")
#let lig = math.op("fi")
#let test = $sech(x) mod_(x -> oo) lig_1(X)$
#test
#show math.op: set text(font: "New Computer Modern")
#test