
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 2-83 failed to decode image (Format error decoding Png: Invalid PNG signature.)
// #image.decode(read("/assets/files/tiger.jpg", encoding: none), format: "png", width: 80%)