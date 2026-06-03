
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Should output `Some` in red, `Some` in blue and `Last` in green.
// Everything should be in smallcaps.
#for color in (red, blue, green, yellow) [
  #set text(font: "Roboto")
  #show: it => text(fill: color, it)
  #smallcaps(if color != green [
    Some
  ] else [
    Last
    #break
  ])
]