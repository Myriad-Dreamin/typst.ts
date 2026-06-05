
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#{
  show table.cell: emph
  table(
    columns: 2,
    [Person], [Animal],
    [John], [Dog]
  )
}