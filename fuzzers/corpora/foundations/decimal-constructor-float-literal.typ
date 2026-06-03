
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Warning: 18-25 creating a decimal using imprecise float literal
// Hint: 18-25 use a string in the decimal constructor to avoid loss of precision: `decimal("1.32523")`
#let _ = decimal(1.32523)