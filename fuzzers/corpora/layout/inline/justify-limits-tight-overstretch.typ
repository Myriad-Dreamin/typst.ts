
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set par(justify: true)
#set text(hyphenate: false)
#let with-limits(..args, body) = {
  set par(justification-limits: args.named())
  body
}

#let it = lorem(3) + linebreak(justify: true)
#let spacer = box(width: 1fr, height: 5pt, fill: aqua)

// Default cost => Spaces are stretched, glyphs are not.
#it

// No adjustments allowed => Spaces are still stretched, but incur
// very high cost while doing so.
#with-limits(
  spacing: (min: 100%, max: 100%),
  tracking: (min: 0em, max: 0em),
  it
)

// Slight tracking is allowed => Spaces are still stretched
// because just tracking is not enough.
#with-limits(
  spacing: (min: 100%, max: 100%),
  tracking: (min: 0em, max: 0.05em),
  it
)

// A ton of tracking is allowed => No space stretching occurs
// anymore.
#with-limits(
  spacing: (min: 100%, max: 150%),
  tracking: (min: 0em, max: 5em),
  it
)

// Test folding against default.
#{
  set par(justification-limits: (tracking: (min: 0em, max: 5em)))
  it
}

// Test folding against a custom value.
#{
  set par(justification-limits: (tracking: (min: 0em, max: 5em)))
  set par(justification-limits: (spacing: (min: 100%, max: 100%)))
  it
}