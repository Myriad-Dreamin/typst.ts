
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// If the hyphen is followed by a space we shall not repeat the hyphen
// at the next line
#set page(width: 4cm)
#set text(lang: "pt", hyphenate: true)

Quebabe é a -melhor- comida que existe.