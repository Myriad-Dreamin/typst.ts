
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(paper: "a10", flipped: true)
#set text(fill: white)
#place(
  dx: -10pt,
  dy: -10pt,
  image(
    "/assets/files/tiger.jpg",
    fit: "cover",
    width: 100% + 20pt,
    height: 100% + 20pt,
  )
)
#align(bottom + right)[
  _Welcome to_ #underline[*Tigerland*]
]
