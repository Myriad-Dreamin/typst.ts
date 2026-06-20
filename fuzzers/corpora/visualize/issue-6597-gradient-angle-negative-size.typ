
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set rect(fill: gradient.linear(angle: 45deg, ..color.map.viridis))
#grid(columns: (1cm, 1cm), rows: 5mm, gutter: 1mm, align: center + horizon,
  rect(width: +1cm, height: +5mm), rect(width: -1cm, height: +5mm),
  rect(width: +1cm, height: -5mm), rect(width: -1cm, height: -5mm),
)