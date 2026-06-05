
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test interaction between RTL and counters
#set text(dir: rtl)
#let test = counter("test")
#grid(
  columns: (1fr, 1fr),
  inset: 5pt,
  align: center,
  grid.cell(rowspan: 2, [
    b: // will produce 2
    #test.step()
    #context test.get().first()
  ]),
  [
    a: // will produce 1
    #test.step()
    #context test.get().first()
  ],
  [
    c: // will produce 3
    #test.step()
    #context test.get().first()
  ]
)