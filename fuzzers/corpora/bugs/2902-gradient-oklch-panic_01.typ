
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 15cm, height: auto, margin: 1em)
#set block(width: 100%, height: 1cm, above: 2pt)

// Oklab
#block(fill: gradient.linear(red, purple, space: oklab))
#block(fill: gradient.linear(..color.map.rainbow, space: oklab))
#block(fill: gradient.linear(..color.map.plasma, space: oklab))
