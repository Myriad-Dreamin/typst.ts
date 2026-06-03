
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Make sure non-breaking and normal space always
// have the same width. Even if the font decided
// differently.
#set text(font: "New Computer Modern")
a b \
a~b