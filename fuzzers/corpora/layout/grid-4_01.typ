
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that fr columns use the correct base.
#grid(
  columns: (1fr,) * 4,
  rows: (1cm,),
  rect(width: 50%, fill: conifer),
  rect(width: 50%, fill: forest),
  rect(width: 50%, fill: conifer),
  rect(width: 50%, fill: forest),
)
