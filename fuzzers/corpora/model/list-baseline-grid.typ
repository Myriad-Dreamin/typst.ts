
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
- #grid(
    inset: 10pt,
    columns: 2,
    [a], [b],
    [c], [d]
  )