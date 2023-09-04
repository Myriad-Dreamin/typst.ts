
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test spacing for set comprehension.
#set page(width: auto)
$ { x in RR | x "is natural" and x < 10 } $
