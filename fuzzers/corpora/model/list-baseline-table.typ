
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
- #table(
    inset: 10pt,
    columns: 2,
    [a], [b],
    [c], [d]
  )

- #table(
    inset: 10pt,
    columns: 2,
    stroke: none,
    [a], [b],
    [c], [d]
  )