

#set text(fill: gradient.linear(red, blue), font: "Open Sans")
#let rainbow(content) = {
  set text(fill: gradient.linear(..color.map.rainbow))
  box(content)
}

This is a gradient on text, but with a #rainbow[twist]!

