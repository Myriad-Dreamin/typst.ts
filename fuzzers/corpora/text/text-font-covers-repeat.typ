
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Repeatedly use the same font.
#set text(font: (
  (name: "Libertinus Serif", covers: regex("[0-9]")),
  "Libertinus Serif"
))

The number 123.