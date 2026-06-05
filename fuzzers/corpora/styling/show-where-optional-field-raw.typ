
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that where selectors also trigger on set rule fields.
#show raw.where(block: false): box.with(
  fill: luma(220),
  inset: (x: 3pt, y: 0pt),
  outset: (y: 3pt),
  radius: 2pt,
)

This is #raw("fn main() {}") some text.