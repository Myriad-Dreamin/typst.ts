
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test stacks with different directions.
#let widths = (
  30pt, 20pt, 40pt, 15pt,
  30pt, 50%, 20pt, 100%,
)

#let shaded(i, w) = {
  let v = (i + 1) * 10%
  rect(width: w, height: 10pt, fill: rgb(v, v, v))
}

#let items = for (i, w) in widths.enumerate() {
  (align(right, shaded(i, w)),)
}

#set page(width: 50pt, margin: 0pt)
#stack(dir: btt, ..items)
