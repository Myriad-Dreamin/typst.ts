
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  show grid.cell: it => (it.x, it.y)
  grid(
    columns: 2,
    inset: 5pt,
    fill: aqua,
    gutter: 3pt,
    [Hello], [World],
    [Sweet], [Home]
  )
}
#{
  show table.cell: it => pad(rest: it.inset)[#(it.x, it.y)]
  table(
    columns: 2,
    gutter: 3pt,
    [Hello], [World],
    [Sweet], [Home]
  )
}