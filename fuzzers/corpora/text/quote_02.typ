
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Spacing with other blocks
#set quote(block: true)

#lorem(10)
#quote(lorem(10))
#lorem(10)
