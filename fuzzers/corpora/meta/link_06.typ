
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Transformed link.
#set page(height: 60pt)
#let mylink = link("https://typst.org/")[LINK]
My cool #box(move(dx: 0.7cm, dy: 0.7cm, rotate(10deg, scale(200%, mylink))))
