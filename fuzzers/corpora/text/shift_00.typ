
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#table(
  columns: 3,
  [Typo.], [Fallb.], [Synth],
  [x#super[1]], [x#super[5n]], [x#super[2 #box(square(size: 6pt))]],
  [x#sub[1]], [x#sub[5n]], [x#sub[2 #box(square(size: 6pt))]],
)
