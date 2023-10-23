
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page


// Test matrix line drawing (augmentation).
#grid(
  columns: 2,
  gutter: 10pt,

  $ mat(10, 2, 3, 4; 5, 6, 7, 8; augment: #3) $,
  $ mat(10, 2, 3, 4; 5, 6, 7, 8; augment: #(-1)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(hline: 2)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(hline: -1)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(hline: 1, vline: 1)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(hline: -2, vline: -2)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(vline: 2, stroke: 1pt + blue)) $,
  $ mat(100, 2, 3; 4, 5, 6; 7, 8, 9; augment: #(vline: -1, stroke: 1pt + blue)) $,
)
