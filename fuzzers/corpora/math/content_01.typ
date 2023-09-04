
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test tables.
$ x := #table(columns: 2)[x][y]/mat(1, 2, 3)
     = #table[A][B][C] $