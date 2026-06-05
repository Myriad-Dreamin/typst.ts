
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that gradient fills on text.
// The solid bar gradients are used to make sure that all transforms are
// correct: if you can see the text through the bar, then the gradient is
// misaligned to its reference container.
#set page(width: 200pt, height: auto, margin: 10pt)
#set par(justify: true)
#set text(fill: gradient.radial(red, blue))
#lorem(30)