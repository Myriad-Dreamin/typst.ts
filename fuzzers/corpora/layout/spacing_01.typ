
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test spacing collapsing before spacing.
#set align(right)
A #h(0pt) B #h(0pt) \
A B \
A #h(-1fr) B
