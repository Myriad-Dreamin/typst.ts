
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test customization.
#show footnote: set text(red)
#show footnote.entry: set text(8pt, style: "italic")
#set footnote.entry(
  indent: 0pt,
  gap: 0.6em,
  clearance: 0.3em,
  separator: repeat[.],
)

Beautiful footnotes. #footnote[Wonderful, aren't they?]
