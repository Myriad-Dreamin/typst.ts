
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let c = counter("c")
#let cd = context c.display()

#set page(
  height: 100pt,
  margin: (y: 20pt),
  header: [H: #cd],
  footer: [F: #cd],
  columns: 2,
)

#let t(align, scope: "column", n) = place(
  align,
  float: true,
  scope: scope,
  clearance: 10pt,
  line(length: 100%) + c.update(n),
)

#t(bottom, 6)
#cd
#t(top, 3)
#colbreak()
#cd
#t(scope: "parent", bottom, 11)
#colbreak()
#cd
#t(top, 12)