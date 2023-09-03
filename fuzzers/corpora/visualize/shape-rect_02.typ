
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Error: 15-38 unexpected key "cake", valid keys are "top-left", "top-right", "bottom-right", "bottom-left", "left", "top", "right", "bottom", and "rest"
// #rect(radius: (left: 10pt, cake: 5pt))