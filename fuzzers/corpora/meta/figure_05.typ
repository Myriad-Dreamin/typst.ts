
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test localized default separator
#set text(lang: "fr", region: "CH")

#figure(
    circle(),
    caption: [Un cercle.],
)
#set text(lang: "es")

#figure(
    polygon.regular(size: 1cm, vertices: 3),
    caption: [Un triángulo.],
)

#set text(lang: "fr", region: "CA")

#figure(
    square(),
    caption: [Un carré.],
)
