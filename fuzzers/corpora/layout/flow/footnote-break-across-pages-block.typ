
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)
#block[
  #lines(3) #footnote(lines(6, "1"))
  #footnote[Y]
  #footnote[Z]
]