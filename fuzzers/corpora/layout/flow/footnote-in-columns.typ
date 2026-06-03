
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 120pt, columns: 2)

#place(
  top + center,
  float: true,
  scope: "parent",
  clearance: 12pt,
  strong[Title],
)

#lines(3)
#footnote(lines(4, "1"))

#lines(2)
#footnote(lines(2, "1"))