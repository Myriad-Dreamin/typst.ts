
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// https://github.com/typst/typst/issues/6186
#let count(n) = lorem(n).replace("–", "").replace(".", "").split(" ").filter(s => s != "").len()
#test(count(193), 193)
#test(count(194), 194)
#test(count(195), 195)