
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test page fill.
#set page(width: 80pt, height: 40pt, fill: eastern)
#text(15pt, font: "Roboto", fill: white, smallcaps[Typst])
#page(width: 40pt, fill: none, margin: (top: 10pt, rest: auto))[Hi]
