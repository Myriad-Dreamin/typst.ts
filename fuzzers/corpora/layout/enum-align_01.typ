
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Enum number alignment should be 'end' by default
1. a
10. b
100. c

#set enum(number-align: start)
1.  a
8.  b
16. c
