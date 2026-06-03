
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that folding is taken into account.
#set text(5pt)
#set text(2em)

#[
  #show text.where(size: 2em): set text(blue)
  2em not blue
]

#[
  #show text.where(size: 10pt): set text(blue)
  10pt blue
]