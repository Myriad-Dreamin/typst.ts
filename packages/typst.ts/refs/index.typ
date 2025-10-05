
#set page(height: auto)

#let points = json("test-points.json");

#set heading(numbering: "1.")

#outline()

#for v in points [
  #page[
    = #v
    #table(
      columns: (1fr, 1fr),
      table.header("SVG", "Canvas"),
      image("renderer/" + v + ".svg.png"), image("renderer/" + v + ".canvas.png"),
    )
  ]
]


