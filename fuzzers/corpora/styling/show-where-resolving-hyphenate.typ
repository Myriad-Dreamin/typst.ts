
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test again that resolving is *not* taken into account.
#set text(hyphenate: auto)

#[
  #show text.where(hyphenate: auto): underline
  Auto
]
#[
  #show text.where(hyphenate: true): underline
  True
]
#[
  #show text.where(hyphenate: false): underline
  False
]