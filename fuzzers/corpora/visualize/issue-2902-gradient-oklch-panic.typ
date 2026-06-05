
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Minimal reproduction of #2902
#set page(width: 15cm, height: auto, margin: 1em)
#set block(width: 100%, height: 1cm, above: 2pt)

// Oklch
#block(fill: gradient.linear(red, purple, space: oklch))
#block(fill: gradient.linear(..color.map.rainbow, space: oklch))
#block(fill: gradient.linear(..color.map.plasma, space: oklch))