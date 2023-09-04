
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test the `colbreak` and `pagebreak` functions.
#set page(height: 1cm, width: 7.05cm, columns: 2)

A
#colbreak()
#colbreak()
B
#pagebreak()
C
#colbreak()
D
