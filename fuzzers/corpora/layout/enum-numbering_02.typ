
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test numbering with closure.
#enum(
  start: 3,
  spacing: 0.65em - 3pt,
  tight: false,
  numbering: n => text(
    fill: (red, green, blue).at(calc.rem(n, 3)),
    numbering("A", n),
  ),
  [Red], [Green], [Blue], [Red],
)
