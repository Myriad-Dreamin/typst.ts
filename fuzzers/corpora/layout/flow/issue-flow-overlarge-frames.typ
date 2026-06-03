
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug, the first line of the second paragraph was on its page alone an
// the rest moved down. The reason was that the second block resulted in
// overlarge frames because the region wasn't finished properly.
#set page(height: 70pt)
#block(lines(3))
#block(lines(5))