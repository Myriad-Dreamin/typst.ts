
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test number type.
#set text(number-type: "old-style")
0123456789 \
#text(number-type: auto)[0123456789]
