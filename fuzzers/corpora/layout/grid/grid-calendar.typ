
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#show grid.cell: it => {
  if it.y == 0 {
    set text(white)
    strong(it)
  } else {
    // For the second row and beyond, we will write the day number for each
    // cell.

    // In general, a cell's index is given by cell.x + columns * cell.y.
    // Days start in the second grid row, so we subtract 1 row.
    // But the first day is day 1, not day 0, so we add 1.
    let day = it.x + 7 * (it.y - 1) + 1
    if day <= 31 {
      // Place the day's number at the top left of the cell.
      // Only if the day is valid for this month (not 32 or higher).
      place(top + left, dx: 2pt, dy: 2pt, text(8pt, red.darken(40%))[#day])
    }
    it
  }
}

#grid(
  fill: (x, y) => if y == 0 { gray.darken(50%) },
  columns: (30pt,) * 7,
  rows: (auto, 30pt),
  // Events will be written at the bottom of each day square.
  align: bottom,
  inset: 5pt,
  stroke: (thickness: 0.5pt, dash: "densely-dotted"),

  [Sun], [Mon], [Tue], [Wed], [Thu], [Fri], [Sat],

  // This event will occur on the first Friday (sixth column).
  grid.cell(x: 5, fill: yellow.darken(10%))[Call],

  // This event will occur every Monday (second column).
  // We have to repeat it 5 times so it occurs every week.
  ..(grid.cell(x: 1, fill: red.lighten(50%))[Meet],) * 5,

  // This event will occur at day 19.
  grid.cell(x: 4, y: 3, fill: orange.lighten(25%))[Talk],

  // These events will occur at the second week, where available.
  grid.cell(y: 2, fill: aqua)[Chat],
  grid.cell(y: 2, fill: aqua)[Walk],
)