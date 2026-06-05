
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  show grid.cell: emph
  grid(
    columns: 2,
    gutter: 3pt,
    [Hello], [World],
    [Sweet], [Italics]
  )
}