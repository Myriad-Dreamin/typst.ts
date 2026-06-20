
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Larger fr destructs smaller fr.
#set page(height: 100pt)
0
#v(1fr, weak: true)
#v(2fr, weak: true) // wins
2
#v(2fr, weak: true)
#v(4fr, weak: true) // wins
6