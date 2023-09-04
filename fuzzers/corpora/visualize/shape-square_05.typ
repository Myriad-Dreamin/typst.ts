
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Size wins over width and height.
// // Error: 09-20 unexpected argument: width
// #square(width: 10cm, height: 20cm, size: 1cm, fill: rgb("eb5278"))