
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto, height: auto, margin: 5pt)
#let test = g => {
  box(width: 100pt, height: 100pt, {
    place(dx: 40pt, dy: 40pt, circle(radius: 10pt, fill: g))
    for i in range(0,12){
      place(dx: 50pt + 12pt*calc.cos(i*30deg), dy: 50pt + 12pt*calc.sin(i*30deg),
        line(length: 36pt, angle: i*30deg, stroke: g + 10pt)
      )
    }
  })
}
#grid(columns: 4,
  ..(0deg, 30deg, 45deg, 90deg, 180deg, -125deg).map(
    a => test(gradient.linear(yellow, black, angle: a).sharp(4))),
  test(gradient.radial(blue, orange).sharp(4)),
  test(gradient.conic(..color.map.spectral).sharp(12)),
)