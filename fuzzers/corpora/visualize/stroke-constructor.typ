
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Converting to stroke
#test(stroke(red).paint, red)
#test(stroke(red).thickness, auto)
#test(stroke(2pt).paint, auto)
#test(stroke((cap: "round", paint: blue)).cap, "round")
#test(stroke((cap: auto, paint: blue)).cap, auto)
#test(stroke((cap: auto, paint: blue)).thickness, auto)

// Constructing with named arguments
#test(stroke(paint: blue, thickness: 8pt), 8pt + blue)
#test(stroke(thickness: 2pt), stroke(2pt))
#test(stroke(cap: "round").thickness, auto)
#test(stroke(cap: "round", thickness: auto).thickness, auto)