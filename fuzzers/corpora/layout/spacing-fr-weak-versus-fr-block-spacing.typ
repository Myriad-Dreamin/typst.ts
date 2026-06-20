
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Weak fr spacing wins against fr block spacing, just like for weak rel
// spacing.
#set page(height: 100pt)
0
#v(1fr, weak: true)
#block(above: 2fr, below: 0pt, height: 0pt)
1
#v(1fr, weak: true)
2