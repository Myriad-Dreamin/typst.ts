
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// 
// // Converting to stroke
// #assert.eq(stroke(red).paint, red)
// #assert.eq(stroke(red).thickness, auto)
// #assert.eq(stroke(2pt).paint, auto)
// #assert.eq(stroke((cap: "round", paint: blue)).cap, "round")
// #assert.eq(stroke((cap: auto, paint: blue)).cap, auto)
// #assert.eq(stroke((cap: auto, paint: blue)).thickness, auto)
// 
// // Error: 9-21 unexpected key "foo", valid keys are "paint", "thickness", "cap", "join", "dash", and "miter-limit"
// #stroke((foo: "bar"))
// 
// // Constructing with named arguments
// #assert.eq(stroke(paint: blue, thickness: 8pt), 8pt + blue)
// #assert.eq(stroke(thickness: 2pt), stroke(2pt))
// #assert.eq(stroke(cap: "round").thickness, auto)
// #assert.eq(stroke(cap: "round", thickness: auto).thickness, auto)