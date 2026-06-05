
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that an outline does not produce paragraphs.
#show par: highlight

#outline()

= A
= B
= C