
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Ensure that radius is clamped.
#rect(radius: -20pt)
#square(radius: 30pt)
