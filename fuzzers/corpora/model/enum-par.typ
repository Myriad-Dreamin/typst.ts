
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Check whether the contents of enum items become paragraphs.
#show par: it => if target() != "html" { highlight(it) } else { it }

// No paragraphs.
#block[
  + Hello
  + World
]

#block[
  + Hello // Paragraphs

    From
  + World // No paragraph because it's a tight enum
]

#block[
  + Hello // Paragraphs

    From

    The

  + World // Paragraph because it's a wide enum
]