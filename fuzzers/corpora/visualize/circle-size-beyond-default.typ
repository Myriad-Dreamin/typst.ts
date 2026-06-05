
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that setting a circle's height beyond its default sizes it correctly.
#circle()
#circle(height: 60pt)
#circle(width: 60pt)
#circle(radius: 30pt)