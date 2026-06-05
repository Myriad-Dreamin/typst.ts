
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let sq = box(square(size: 4pt))
#table(
  columns: 3,
  [Typo.], [Fallb.], [Synth.],
  [x#super[1#sq]], [x#super[5: #sq]], [x#super(typographic: false)[2 #sq]],
  [x#sub[1#sq]], [x#sub[5: #sq]], [x#sub(typographic: false)[2 #sq]],
)