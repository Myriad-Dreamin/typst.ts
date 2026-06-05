
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 60pt, height: 140pt)
#set text(weight: 700)

// Fits fully onto the first page.
#set text(blue)
#lines(8)

// The first line would fit, but is moved to the second page.
#lines(6, "1")

// The second-to-last line is moved to the third page so that the last is isn't
// as lonely.
#set text(maroon)
#lines(4)

#lines(4, "1")

// All three lines go to the next page.
#set text(olive)
#lines(3)