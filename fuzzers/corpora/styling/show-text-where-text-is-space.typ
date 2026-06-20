
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Spaces in `text` match even if differently styled, unlike the previous test.
#show text.where(text: " "): [B]
A#text(" ")C \
A#text([ ])C \ // Space elements don't magically become `text`
A#text(" ", red)C