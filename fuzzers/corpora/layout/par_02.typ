
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that paragraph spacing loses against block spacing.
// TODO
#set block(spacing: 100pt)
#show table: set block(above: 5pt, below: 5pt)
Hello
#table(columns: 4, fill: (x, y) => if calc.odd(x + y) { silver })[A][B][C][D]
