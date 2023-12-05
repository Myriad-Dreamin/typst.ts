
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Spacing with other blocks
#set quote(block: true)
#set text(8pt)

#lorem(10)
#quote(lorem(10))
#lorem(10)
