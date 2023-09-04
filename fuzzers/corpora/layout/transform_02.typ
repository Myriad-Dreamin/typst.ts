
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test setting rotation origin.
#rotate(10deg, origin: top + left,
  image("/assets/files/tiger.jpg", width: 50%)
)
