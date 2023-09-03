
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 3cm, margin: 0pt)
#grid(
  columns: (1fr,),
  rows: (1fr, auto, 2fr),
  [],
  align(center)[A bit more to the top],
  [],
)
