
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 3-37 cannot draw a vertical line after column 3 of a matrix with 3 columns
// $ mat(1, 0, 0; 0, 1, 1; augment: #3) $,