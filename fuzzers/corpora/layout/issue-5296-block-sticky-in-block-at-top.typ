
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 3cm)
#v(1.6cm)
#block(height: 2cm, breakable: true)[
  #block(sticky: true)[*A*]

  b
]