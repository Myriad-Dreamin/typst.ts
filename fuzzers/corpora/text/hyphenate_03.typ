
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test shy hyphens.
#set text(lang: "de", hyphenate: true)
#grid(
  columns: 2 * (20pt,),
  gutter: 20pt,
  [Barankauf],
  [Bar-?ankauf],
)
