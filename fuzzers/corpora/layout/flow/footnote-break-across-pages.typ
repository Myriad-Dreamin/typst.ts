
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 200pt)

#lines(2)
#footnote[ // 1
  I
  #footnote[II ...] // 2
]
#lines(6)
#footnote[III: #lines(8, "1")] // 3
#lines(6)
#footnote[IV: #lines(15, "1")] // 4
#lines(6)
#footnote[V] // 5