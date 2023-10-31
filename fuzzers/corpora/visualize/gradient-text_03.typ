
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that gradient fills on text work for locally defined gradients.

#set page(width: auto, height: auto, margin: 10pt)
#show box: set text(fill: gradient.linear(..color.map.rainbow))

Hello, #box[World]!
