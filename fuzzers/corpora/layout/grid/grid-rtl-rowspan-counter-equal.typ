
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
    a: // should produce 1
    #test.step()
    #context test.get().first()
  ]),
  grid.cell(rowspan: 2, [
    b: // should produce 2
    #test.step()
    #context test.get().first()
  ]),
)