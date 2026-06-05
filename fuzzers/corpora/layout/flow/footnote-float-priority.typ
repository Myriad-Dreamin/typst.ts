
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)

#lines(3)

#place(
  top,
  float: true,
  rect(height: 40pt)
)

#block[
  V
  #footnote[1]
  #footnote[2]
  #footnote[3]
  #footnote[4]
]

#lines(5)