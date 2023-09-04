
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that lines with hard breaks aren't justified.
#set par(justify: true)
A B C \
D
