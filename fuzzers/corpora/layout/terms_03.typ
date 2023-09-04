
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test style change.
#set text(8pt)
/ First list: #lorem(6)

#set terms(hanging-indent: 30pt)
/ Second list: #lorem(5)
