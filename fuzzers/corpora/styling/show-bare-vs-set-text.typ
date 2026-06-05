
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test style precedence.
#set text(fill: eastern, size: 1.5em)
#show: text.with(fill: forest)
Forest