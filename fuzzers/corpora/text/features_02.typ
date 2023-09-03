
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test alternates and stylistic sets.
#set text(font: "IBM Plex Serif")
a vs #text(alternates: true)[a] \
ß vs #text(stylistic-set: 5)[ß]
