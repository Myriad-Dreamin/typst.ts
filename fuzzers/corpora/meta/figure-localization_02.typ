
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test Russian
#set text(lang: "ru")

#figure(
    polygon.regular(size: 1cm, vertices: 8),
    caption: [Пятиугольник],
)
