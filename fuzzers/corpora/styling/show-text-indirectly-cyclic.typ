
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test indirect cycle.
#show "Good": [Typst!]
#show "Typst": [Fun!]
#show "Fun": [Good!]

#set text(ligatures: false)
Good \
Fun \
Typst \