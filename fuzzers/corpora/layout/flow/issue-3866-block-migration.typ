
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 120pt)
#set text(costs: (widow: 0%, orphan: 0%))
#v(50pt)
#columns(2)[
  #lines(6)
  #block(rect(width: 80%, height: 80pt), breakable: false)
  #lines(6)
]