
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test box sizing with layoutable child.
#box(
  width: 50pt,
  height: 50pt,
  fill: yellow,
  path(
    fill: purple,
    (0pt, 0pt),
    (30pt, 30pt),
    (0pt, 30pt),
    (30pt, 0pt),
  ),
)
