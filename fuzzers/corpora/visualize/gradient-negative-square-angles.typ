
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#grid(columns: 12, ..for i in range(-24,24){(
  rect(width: 3mm, height: 3mm, fill: gradient.linear(yellow, black, angle: i * 15deg).sharp(3)),
)})