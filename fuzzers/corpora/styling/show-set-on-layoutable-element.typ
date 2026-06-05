
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test show-set rules on layoutable element to ensure it is realized
// even though it implements `LayoutMultiple`.
#show table: set text(red)
#pad(table(columns: 4)[A][B][C][D])