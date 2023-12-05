
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 29-55 expected "solid", "dotted", "densely-dotted", "loosely-dotted", "dashed", "densely-dashed", "loosely-dashed", "dash-dotted", "densely-dash-dotted", "loosely-dash-dotted", array, dictionary, none, or auto
// #line(length: 60pt, stroke: (paint: red, dash: "dash"))