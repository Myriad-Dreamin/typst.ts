
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#box(fill: aqua)[
  #place(top + left, dx: 50%, dy: 50%)[Hi]
  #v(30pt)
  #line(length: 50pt)
]
