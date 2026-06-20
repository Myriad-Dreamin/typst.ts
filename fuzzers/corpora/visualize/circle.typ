
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Default circle.
#stack(
  dir: ltr,
  spacing: 0.5em,
  circle(),
  circle[Hey]
)