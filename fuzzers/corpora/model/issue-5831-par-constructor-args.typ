
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Make sure that all arguments are also respected in the constructor.
A
#par(
  leading: 2pt,
  spacing: 20pt,
  justify: true,
  linebreaks: "simple",
  first-line-indent: (amount: 1em, all: true),
  hanging-indent: 5pt,
)[
  The par function has a constructor and justification.
]