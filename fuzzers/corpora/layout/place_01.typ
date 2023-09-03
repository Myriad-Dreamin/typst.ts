
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test how the placed element interacts with paragraph spacing around it.
#set page("a8", height: 60pt)

First

#place(bottom + right)[Placed]

Second
