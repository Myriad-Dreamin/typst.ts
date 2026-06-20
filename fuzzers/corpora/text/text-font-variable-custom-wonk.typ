
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#set text(font: "Fraunces", size: 25pt)

// WONK only kicks in at point sizes > 18pt.
#text(variations: (WONK: 0))[minimum] \
minimum \
#text(variations: (WONK: 1))[minimum]