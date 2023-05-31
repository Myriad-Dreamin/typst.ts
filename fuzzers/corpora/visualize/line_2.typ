// Test lines.

// Test the angle argument and positioning.

#set page(fill: rgb("0B1026"), width: auto, height: auto)
#set line(stroke: white)

#let star(size, ..args) = box(width: size, height: size)[
  #set text(spacing: 0%)
  #set line(..args)
  #set align(left)
  #v(30%)
  #place(line(length: +30%, start: (09.0%, 02%)))
  #place(line(length: +30%, start: (38.7%, 02%), angle: -72deg))
  #place(line(length: +30%, start: (57.5%, 02%), angle: 252deg))
  #place(line(length: +30%, start: (57.3%, 02%)))
  #place(line(length: -30%, start: (88.0%, 02%), angle: -36deg))
  #place(line(length: +30%, start: (73.3%, 48%), angle: 252deg))
  #place(line(length: -30%, start: (73.5%, 48%), angle: 36deg))
  #place(line(length: +30%, start: (25.4%, 48%), angle: -36deg))
  #place(line(length: +30%, start: (25.6%, 48%), angle: -72deg))
  #place(line(length: +32%, start: (8.50%, 02%), angle: 34deg))
]

#align(center, grid(
  columns: 3,
  column-gutter: 10pt,
  ..((star(20pt, stroke: 0.5pt),) * 9)
))
