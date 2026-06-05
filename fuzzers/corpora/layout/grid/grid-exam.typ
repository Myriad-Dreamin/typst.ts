
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#show table.cell: it => {
  if it.x == 0 or it.y == 0 {
    set text(white)
    strong(it)
  } else if it.body == [] {
    // Replace empty cells with 'N/A'
    pad(rest: it.inset)[_N/A_]
  } else {
    it
  }
}

#table(
  fill: (x, y) => if x == 0 or y == 0 { gray.darken(50%) },
  columns: 4,
  [], [Exam 1], [Exam 2], [Exam 3],
  ..([John], [Mary], [Jake], [Robert]).map(table.cell.with(x: 0)),

  // Mary got grade A on Exam 3.
  table.cell(x: 3, y: 2, fill: green)[A],

  // Everyone got grade A on Exam 2.
  ..(table.cell(x: 2, fill: green)[A],) * 4,

  // Robert got grade B on other exams.
  ..(table.cell(y: 4, fill: aqua)[B],) * 2,
)