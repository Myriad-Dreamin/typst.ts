
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Soft axis becomes more visible at large font size, so we increase it and then
// scale down to avoid a huge test image.
#set text(font: "Fraunces", size: 100pt)
#scale(20%, reflow: true)[
  #set text(variations: (SOFT: 0))
  Soft?
  #set text(variations: (SOFT: 100))
  Soft!
]