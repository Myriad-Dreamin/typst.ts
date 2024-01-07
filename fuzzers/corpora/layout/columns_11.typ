
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test colbreak after only out-of-flow elements.
#set page(width: 7.05cm, columns: 2)
#place[OOF]
#colbreak()
In flow.
