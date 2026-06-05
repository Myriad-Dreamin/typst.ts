
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that placed elements don't add extra block spacing.
#show figure: set block(spacing: 4em)

Paragraph before float.
#figure(rect(), placement: bottom)
Paragraph after float.