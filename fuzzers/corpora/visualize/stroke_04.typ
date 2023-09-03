
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Line joins
#stack(
  dir: ltr,
  spacing: 1em,
  polygon(stroke: (thickness: 4pt, paint: blue, join: "round"),
    (0pt, 20pt), (15pt, 0pt), (0pt, 40pt), (15pt, 45pt)),
  polygon(stroke: (thickness: 4pt, paint: blue, join: "bevel"),
    (0pt, 20pt), (15pt, 0pt), (0pt, 40pt), (15pt, 45pt)),
  polygon(stroke: (thickness: 4pt, paint: blue, join: "miter"),
    (0pt, 20pt), (15pt, 0pt), (0pt, 40pt), (15pt, 45pt)),
  polygon(stroke: (thickness: 4pt, paint: blue, join: "miter", miter-limit: 20.0),
    (0pt, 20pt), (15pt, 0pt), (0pt, 40pt), (15pt, 45pt)),
)