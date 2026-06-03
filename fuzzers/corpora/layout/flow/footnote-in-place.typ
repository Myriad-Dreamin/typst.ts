
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
A
#place(top + right, footnote[A])
#figure(
  placement: bottom,
  caption: footnote[B],
  rect(),
)