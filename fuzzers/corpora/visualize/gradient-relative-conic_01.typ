
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// The image should look as if there are multiple gradients, one for each
// rectangle.
#let grad = gradient.conic(red, blue, green, purple, relative: "self");
#let my-rect = rect(width: 50%, height: 50%, fill: grad)
#set page(
  height: 200pt,
  width: 200pt,
  fill: grad,
  background: place(top + left, my-rect),
)
#place(top + right, my-rect)
#place(bottom + center, rotate(45deg, my-rect))
