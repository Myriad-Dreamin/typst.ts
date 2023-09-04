
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 29-56 unexpected key "thicknes", valid keys are "paint", "thickness", "cap", "join", "dash", and "miter-limit"
// #line(length: 60pt, stroke: (paint: red, thicknes: 1pt))