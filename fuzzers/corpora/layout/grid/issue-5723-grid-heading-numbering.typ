
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set heading(numbering: "1.1.")
#set page(width: 150pt, height: 3.5cm)

#table(
  columns: (1fr, 2fr),
  [= A],
  [= B],
  [
    = C
    #lines(4)
    = D
  ],
  table(
    columns: (1fr, 1fr),
    ..([
      = X
      #lines(2)
      = Y
      #lines(2)
    ],) * 2
  ),
  [= E],
  [= F]
)