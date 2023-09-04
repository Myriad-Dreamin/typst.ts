
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test string body.
#text("Text") \
#text(red, "Text") \
#text(font: "Ubuntu", blue, "Text") \
#text([Text], teal, font: "IBM Plex Serif") \
#text(forest, font: "New Computer Modern", [Text]) \
