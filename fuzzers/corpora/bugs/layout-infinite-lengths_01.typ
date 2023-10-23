
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// #set page(width: auto, height: auto)
// 
// // Error: 17-66 cannot create grid with infinite height
// #layout(size => grid(rows: (size.width, size.height))[a][b][c][d])