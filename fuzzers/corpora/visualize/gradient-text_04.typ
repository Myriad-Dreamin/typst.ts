
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that gradients fills on text work with transforms.

#set page(width: auto, height: auto, margin: 10pt)
#show box: set text(fill: gradient.linear(..color.map.rainbow))

#rotate(45deg, box[World])
