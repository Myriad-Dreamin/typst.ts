// SKIP: Temporarily removed for Typst 0.15.0-rc1 corpus compatibility review.

#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test box in paragraph.
A #box[B \ C] D.

// Test box with height.
Spaced \
#box(height: 0.5cm) \
Apart