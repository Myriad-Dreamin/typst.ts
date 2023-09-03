
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Radius wins over width and height.
// // Error: 23-34 unexpected argument: width
// #circle(radius: 10pt, width: 50pt, height: 100pt, fill: eastern)