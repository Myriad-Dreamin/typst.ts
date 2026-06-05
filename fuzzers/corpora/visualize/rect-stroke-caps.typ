
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Separated segments
#rect(width: 20pt, height: 20pt, stroke: (
  left: (cap: "round", thickness: 5pt),
  right: (cap: "square", thickness: 7pt),
))
// Joined segment with different caps.
#rect(width: 20pt, height: 20pt, stroke: (
  left: (cap: "round", thickness: 5pt),
  top: (cap: "square", thickness: 7pt),
))
// No caps when there is a radius for that corner.
#rect(width: 20pt, height: 20pt, radius: (top: 3pt), stroke: (
  left: (cap: "round", thickness: 5pt),
  top: (cap: "square", thickness: 7pt),
))