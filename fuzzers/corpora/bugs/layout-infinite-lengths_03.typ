
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// #set page(width: auto, height: auto)
// 
// // Error: 17-54 cannot create polygon with infinite size
// #layout(size => polygon((0pt,0pt), (0pt, size.width)))