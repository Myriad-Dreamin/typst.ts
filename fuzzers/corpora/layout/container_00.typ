
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test box in paragraph.
A #box[B \ C] D.

// Test box with height.
Spaced \
#box(height: 0.5cm) \
Apart
