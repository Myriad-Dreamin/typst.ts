
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// #set page(width: auto, height: auto)
// 
// // Error: cannot expand into infinite width
// #layout(size => grid(columns: (size.width, size.height))[a][b][c][d])