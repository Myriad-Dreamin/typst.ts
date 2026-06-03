
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 120pt)
#let floater(align, height) = place(
  align,
  float: true,
  rect(width: 100%, height: height),
)

#floater(top, 30pt)
A

#floater(bottom, 50pt)
#place.flush()
B // Should be on the second page.