
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that broken cell expands vertically.
#set page(height: 2.25cm)
#grid(
  columns: 2,
  gutter: 10pt,
  align(bottom)[A],
  [
    Top
    #align(bottom)[
      Bottom \
      Bottom \
      #v(0pt)
      Top
    ]
  ],
  align(top)[B],
)
