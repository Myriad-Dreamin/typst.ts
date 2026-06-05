
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Check whether the contents of term list items become paragraphs.
#show par: it => if target() != "html" { highlight(it) } else { it }

// No paragraphs.
#block[
  / Hello: A
  / World: B
]

#block[
  / Hello: A // Paragraphs

    From
  / World: B // No paragraphs because it's a tight term list.
]

#block[
  / Hello: A // Paragraphs

    From

    The

  / World: B // Paragraph because it's a wide term list.
]