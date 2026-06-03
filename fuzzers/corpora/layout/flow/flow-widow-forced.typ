
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that a widow is allowed when the three lines don't all fit.
#set page(height: 50pt)
#lines(3)