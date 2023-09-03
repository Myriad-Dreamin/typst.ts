
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Ensure circle directly in rect works.
#rect(width: 40pt, height: 30pt, fill: forest,
  circle(fill: conifer))
