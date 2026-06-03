
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that constructor styles aren't passed down the tree.
// The inner list should have no extra indent.
#set par(leading: 2pt)
#list(body-indent: 20pt, [First], list[A][B])