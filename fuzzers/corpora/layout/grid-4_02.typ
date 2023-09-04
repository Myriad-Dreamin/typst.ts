
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that all three kinds of rows use the correct bases.
#set page(height: 4cm, margin: 0cm)
#grid(
  rows: (1cm, 1fr, 1fr, auto),
  rect(height: 50%, width: 100%, fill: conifer),
  rect(height: 50%, width: 100%, fill: forest),
  rect(height: 50%, width: 100%, fill: conifer),
  rect(height: 25%, width: 100%, fill: forest),
)
