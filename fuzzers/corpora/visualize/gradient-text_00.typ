
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Ref: false
// // Make sure they don't work when `relative: "self"`.
// 
// // Hint: 17-61 make sure to set `relative: auto` on your text fill
// // Error: 17-61 gradients on text must be relative to the parent
// #set text(fill: gradient.linear(red, blue, relative: "self"))