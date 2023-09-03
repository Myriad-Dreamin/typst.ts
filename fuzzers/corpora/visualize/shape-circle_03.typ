
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test relative sizing.
#set text(fill: white)
#show: rect.with(width: 100pt, height: 50pt, inset: 0pt, fill: rgb("aaa"))
#set align(center + horizon)
#stack(
  dir: ltr,
  spacing: 1fr,
  1fr,
  circle(radius: 10pt, fill: eastern, [A]),      // D=20pt
  circle(height: 60%, fill: eastern, [B]),       // D=30pt
  circle(width: 20% + 20pt, fill: eastern, [C]), // D=40pt
  1fr,
)
