// Test transformations.

#set page(width: auto, height: auto, margin: 10pt)

// Test combination of scaling and rotation.
#set page(height: 80pt)
#align(center + horizon,
  rotate(20deg, scale(70%, image("tiger.jpg")))
)
