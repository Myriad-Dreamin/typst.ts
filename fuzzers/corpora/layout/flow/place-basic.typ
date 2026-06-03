
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page("a8")
#place(bottom + center)[E]

= A
#place(right, rect(width: 1.8cm))
#lines(5)

#stack(
  rect(fill: eastern, height: 10pt, width: 100%),
  place(right, dy: 1.5pt)[ABC],
  rect(fill: conifer, height: 10pt, width: 80%),
  rect(fill: forest, height: 10pt, width: 100%),
  10pt,
  block[
    #place(center, dx: -7pt, dy: -5pt)[A]
    #place(center, dx: 7pt, dy: 5pt)[B]
    C #h(1fr) D
  ]
)