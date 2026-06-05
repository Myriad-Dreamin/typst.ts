
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure this case (handled separately internally) is properly handled.
#set page(margin: (x: 1.1cm), height: 2cm, columns: 2)
#set par.line(
  numbering: "1",
  number-clearance: 0.5cm,
  numbering-scope: "page"
)

First line
#colbreak()
Second line
#pagebreak()
#place[]
#box(height: 2cm)[First!]