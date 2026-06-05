
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that heading text isn't considered a paragraph.
#show par: highlight
= Heading