
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Change font only for numbers.
#set text(font: (
  (name: "PT Sans", covers: regex("[0-9]")),
  "Libertinus Serif"
))

The number 123.