
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test shape fill & stroke for specific values that used to make the stroke
// deformed.
#rect(
  radius: 1mm,
  width: 100%,
  height: 10pt,
  stroke: (left: rgb("46b3c2") + 16.0mm),
)