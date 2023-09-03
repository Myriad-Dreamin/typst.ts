
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that the last line can be shrunk
#set page(width: 155pt)
#set par(justify: true)
This text can be fitted in one line.
