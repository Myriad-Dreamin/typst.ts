
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#set text(font: "Fraunces")

#for base in (10pt, 20pt) {
  for s in range(1, 5, inclusive: true) [
    #let scaled = s * base
    #scale(100% / s, reflow: true, text(size: scaled)[Hello])
  ]
}