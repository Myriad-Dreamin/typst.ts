
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that custom variations win over built-in settings.
#set text(font: "Mona Sans")
#text(style: "italic")[
  Italic \
  #text(variations: (ital: 0))[Not italic]
]