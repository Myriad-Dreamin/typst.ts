
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test relative path resolving in layout phase.
#let choice = ("monkey.svg", "rhino.png", "tiger.jpg")
#set enum(numbering: n => {
  let path = "/assets/images/" + choice.at(n - 1)
  move(dy: -0.15em, image(path, width: 1em, height: 1em))
})

+ Monkey
+ Rhino
+ Tiger