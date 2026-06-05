
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that a figure body is considered a paragraph.
#show par: highlight

#figure[Text]

#figure(
  [Text],
  caption: [A caption]
)