
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test box/block sizing with directly layoutable child.
//
// Ensure that the output respects the box size.
#let check(f) = f(
  width: 40pt, height: 25pt, fill: aqua,
  grid(rect(width: 5pt, height: 5pt, fill: blue)),
)

#stack(dir: ltr, spacing: 1fr, check(box), check(block))