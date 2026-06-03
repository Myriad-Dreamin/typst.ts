
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set text(size: 20pt)
#set page(width: auto)
#let v = [测试字体Test]

#text(stroke: 0.3pt + red, v)

#text(stroke: 0.7pt + red, v)

#text(stroke: 7pt + red, v)

#text(stroke: (paint: blue, thickness: 1pt, dash: "dashed"), v)

#text(stroke: 1pt + gradient.linear(..color.map.rainbow), v)