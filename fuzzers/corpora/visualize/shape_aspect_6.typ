// Test that squares and circles respect their 1-1 aspect ratio.

// Test that square doesn't overflow due to its aspect ratio.
#set page(width: 40pt, height: 25pt, margin: 5pt)
#square(width: 100%)
#square(width: 100%)[Hello there]
