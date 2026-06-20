
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Default square.
#stack(
  dir: ltr,
  spacing: 0.5em,
  square(),
  square[hey!]
)