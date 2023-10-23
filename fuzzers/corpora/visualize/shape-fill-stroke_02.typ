
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test stroke composition.
#set square(stroke: 4pt)
#set text(font: "Roboto")
#stack(
  dir: ltr,
  square(
    stroke: (left: red, top: yellow, right: green, bottom: blue),
    radius: 50%, align(center+horizon)[*G*],
    inset: 8pt
  ),
  h(0.5cm),
  square(
    stroke: (left: red, top: yellow + 8pt, right: green, bottom: blue + 2pt),
    radius: 50%, align(center+horizon)[*G*],
    inset: 8pt
  ),
  h(0.5cm),
  square(
    stroke: (left: red, top: yellow, right: green, bottom: blue),
    radius: 100%, align(center+horizon)[*G*],
    inset: 8pt
  ),
)

// Join between different solid strokes
#set square(size: 20pt, stroke: 2pt)
#set square(stroke: (left: green + 4pt, top: black + 2pt, right: blue, bottom: black + 2pt))
#stack(
  dir: ltr,
  square(),
  h(0.2cm),
  square(radius: (top-left: 0pt, rest: 1pt)),
  h(0.2cm),
  square(radius: (top-left: 0pt, rest: 8pt)),
  h(0.2cm),
	square(radius: (top-left: 0pt, rest: 100pt)),
)


// Join between solid and dotted strokes
#set square(stroke: (left: green + 4pt, top: black + 2pt, right: (paint: blue, dash: "dotted"), bottom: (paint: black, dash: "dotted")))
#stack(
  dir: ltr,
  square(),
  h(0.2cm),
  square(radius: (top-left: 0pt, rest: 1pt)),
  h(0.2cm),
  square(radius: (top-left: 0pt, rest: 8pt)),
  h(0.2cm),
	square(radius: (top-left: 0pt, rest: 100pt)),
)
