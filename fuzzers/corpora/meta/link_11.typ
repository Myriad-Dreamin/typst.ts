
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// Text <hey>
// Text <hey>
// // Error: 2-20 label `<hey>` occurs multiple times in the document
// #link(<hey>)[Nope.]