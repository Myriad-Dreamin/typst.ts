
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(
  first-line-indent: (amount: 12pt, all: true),
  spacing: 5pt,
  leading: 5pt,
)
#set block(spacing: 1.2em)
#show heading: set text(size: 10pt)

= Heading
All paragraphs are indented.

Even the first.