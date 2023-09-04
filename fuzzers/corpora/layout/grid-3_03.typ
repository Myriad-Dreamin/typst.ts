
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test grid within a grid, overflowing.
#set page(width: 5cm, height: 2.25cm)
#grid(
  columns: 4 * (1fr,),
  row-gutter: 10pt,
  column-gutter: (0pt, 10%),
  [A], [B], [C], [D],
  grid(columns: 2, [A], [B], [C\ ]*3, [D]),
  align(top, rect(inset: 0pt, fill: eastern, align(right)[LoL])),
  [rofl],
  [E\ ]*4,
)
