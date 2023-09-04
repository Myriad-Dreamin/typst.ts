
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test images and font fallback.
#let monkey = move(dy: 0.2em, image("/assets/files/monkey.svg", height: 1em))
$ sum_(i=#emoji.apple)^#emoji.apple.red i + monkey/2 $
