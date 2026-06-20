
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let content = place(dx: 5pt, dy: 5pt, circle(radius: 5pt, fill: blue))
#let pat1 = tiling(size: (20pt, 20pt), content)
// Second tiling, only the size attribute changes
#let pat2 = tiling(size: (40pt, 20pt), content)

#rect(fill: pat1, width: 100pt, height: 20pt, stroke: 1pt)
#rect(fill: pat2, width: 100pt, height: 20pt, stroke: 1pt)