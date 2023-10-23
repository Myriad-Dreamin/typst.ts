
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#show figure: set block(spacing: 4em)

Paragraph before float.
#figure(rect(), placement: bottom)
Paragraph after float.
