
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test a column that starts overflowing right after another row/column did
// that.
#set page(width: 5cm, height: 2cm)
#grid(
  columns: 4 * (1fr,),
  row-gutter: 10pt,
  column-gutter: (0pt, 10%),
  align(top, image("/assets/files/rhino.png")),
  align(top, rect(inset: 0pt, fill: eastern, align(right)[LoL])),
  [rofl],
  [\ A] * 3,
  [Ha!\ ] * 3,
)
