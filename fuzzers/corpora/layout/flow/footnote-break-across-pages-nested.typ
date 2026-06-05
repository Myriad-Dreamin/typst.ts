
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 120pt)
#block[
  #lines(4)
  #footnote[
    #lines(6, "1")
    #footnote(lines(3, "I"))
  ]
]