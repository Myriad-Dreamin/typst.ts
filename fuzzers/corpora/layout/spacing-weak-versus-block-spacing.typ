
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Weak spacing wins against block spacing.
0
#v(1cm, weak: true)
#block(above: 2cm, below: 0pt, height: 0pt)
1
#v(1cm, weak: true)
2