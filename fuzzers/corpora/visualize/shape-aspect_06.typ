
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Size cannot be relative because we wouldn't know
// // relative to which axis.
// // Error: 15-18 expected length or auto, found ratio
// #square(size: 50%)