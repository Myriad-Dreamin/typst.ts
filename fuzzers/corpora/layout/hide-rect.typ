
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set rect(
  inset: 8pt,
  fill: rgb("e4e5ea"),
  width: 100%,
)

Hidden:
#hide[
#grid(
  columns: (1fr, 1fr, 2fr),
  rows: (auto, 40pt),
  gutter: 3pt,
  rect[A],
  rect[B],
  rect[C],
  rect(height: 100%)[D],
)
]
#grid(
  columns: (1fr, 1fr, 2fr),
  rows: (auto, 40pt),
  gutter: 3pt,
  rect[A],
  rect[B],
  rect[C],
  rect(height: 100%)[D],
)