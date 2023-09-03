
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let variant = rect.with(width: 20pt, height: 10pt)
#let items = for (i, item) in (
  variant(stroke: none),
  variant(),
  variant(fill: none),
  variant(stroke: 2pt),
  variant(stroke: eastern),
  variant(stroke: eastern + 2pt),
  variant(fill: eastern),
  variant(fill: eastern, stroke: none),
  variant(fill: forest, stroke: none),
  variant(fill: forest, stroke: conifer),
  variant(fill: forest, stroke: black + 2pt),
  variant(fill: forest, stroke: conifer + 2pt),
).enumerate() {
  (align(horizon)[#(i + 1).], item, [])
}

#grid(
  columns: (auto, auto, 1fr, auto, auto, 0fr),
  gutter: 5pt,
  ..items,
)
