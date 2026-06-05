
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The image should look as if there are two nested gradients, one for the page
// and one for a nested block. The rotated rectangles are not visible because
// they are relative to the block.
#let grad = gradient.linear(red, blue, green, purple, relative: "parent")
#let my-rect = rect(width: 50%, height: 50%, fill: grad)
#set page(
  height: 50pt,
  width: 50pt,
  margin: 5pt,
  fill: grad,
  background: place(top + left, my-rect),
)
#block(
  width: 40pt,
  height: 40pt,
  inset: 2.5pt,
  fill: grad,
)[
  #place(top + right, my-rect)
  #place(bottom + center, rotate(45deg, my-rect))
]