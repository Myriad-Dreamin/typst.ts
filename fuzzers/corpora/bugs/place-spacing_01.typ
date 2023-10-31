
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#show place: set block(spacing: 4em)

Paragraph before place.
#place(rect())
Paragraph after place.
