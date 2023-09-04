
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test number width.
#text(number-width: "proportional")[0123456789] \
#text(number-width: "tabular")[3456789123] \
#text(number-width: "tabular")[0123456789]
