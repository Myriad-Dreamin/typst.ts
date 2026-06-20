
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set text(variations: (ital: 1, GRAD: 10))
#set text(variations: (GRAD: 15))
#context test(text.variations, (ital: 1, GRAD: 15))