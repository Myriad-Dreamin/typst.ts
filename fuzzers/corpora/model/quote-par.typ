
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that an inline quote is part of a paragraph, but a block quote
// does not result in paragraphs.
#show par: highlight

An inline #quote[quote.]

#quote(block: true, attribution: [The Test Author])[
  A block-level quote.
]