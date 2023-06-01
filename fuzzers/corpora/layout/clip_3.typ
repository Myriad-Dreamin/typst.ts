// Test clipping with the `box` and `block` containers.

#set page(width: 120pt, height: auto, margin: 10pt)

// Test cliping svg glyphs
Emoji: #box(height: 0.5em, stroke: 1pt + black)[🐪, 🌋, 🏞]

Emoji: #box(height: 0.5em, clip: true, stroke: 1pt + black)[🐪, 🌋, 🏞]