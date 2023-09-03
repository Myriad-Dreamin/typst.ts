
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that multiple paragraphs in subflow also respect alignment.
#align(center)[
  Lorem Ipsum

  Dolor
]
