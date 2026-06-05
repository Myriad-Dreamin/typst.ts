
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test breaking a line without justifiables.
#set par(justify: true)
#block(width: 1cm, fill: aqua, lorem(2))