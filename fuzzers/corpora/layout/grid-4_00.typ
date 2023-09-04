
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that auto and relative columns use the correct base.
#grid(
  columns: (auto, 60%),
  rows: (auto, auto),
  rect(width: 50%, height: 0.5cm, fill: conifer),
  rect(width: 100%, height: 0.5cm, fill: eastern),
  rect(width: 50%, height: 0.5cm, fill: forest),
)
