
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test label after expression.
#show strong.where(label: <v>): set text(red)

#let a = [*A*]
#let b = [*B*]
#a <v> #b