
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(numbering: "1", margin: (bottom: 20pt))
A
#{
  set text(fill: red)
  pagebreak()
}
#text(fill: blue)[B]