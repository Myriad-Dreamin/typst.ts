
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that a footnote and the first line of its entry
// always end up on the same page.
#set page(height: 120pt)

#lines(5)

A #footnote(lines(6, "1"))