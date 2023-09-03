
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test an empty second column.
#set page(width: 7.05cm, columns: 2)

#rect(width: 100%, inset: 3pt)[So there isn't anything in the second column?]
