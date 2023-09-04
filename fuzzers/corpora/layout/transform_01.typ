
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test combination of scaling and rotation.
#set page(height: 80pt)
#align(center + horizon,
  rotate(20deg, scale(70%, image("/assets/files/tiger.jpg")))
)
