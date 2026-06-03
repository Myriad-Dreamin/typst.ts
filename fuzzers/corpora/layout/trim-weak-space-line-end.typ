
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Weak space at the end of the line should be removed.
#set align(right)
Hello #h(2cm, weak: true)