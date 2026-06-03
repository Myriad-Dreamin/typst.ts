
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 180pt)

#lines(5)

#place(
  bottom,
  float: true,
  rect(height: 50pt, width: 100%, {
    footnote(lines(6, "1"))
    footnote(lines(2, "I"))
  })
)

#lines(5)