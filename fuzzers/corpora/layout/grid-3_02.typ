
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test two columns in the same row overflowing by a different amount.
#set page(width: 5cm, height: 2cm)
#grid(
  columns: 3 * (1fr,),
  row-gutter: 8pt,
  column-gutter: (0pt, 10%),
  [A], [B], [C],
  [Ha!\ ] * 6,
  [rofl],
  [\ A] * 3,
  [hello],
  [darkness],
  [my old]
)
