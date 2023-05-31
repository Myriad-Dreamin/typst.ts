// Test text decorations.

#set page(height: auto, width: 150pt, margin: 10pt)

#let redact = strike.with(stroke: 10pt, extent: 0.05em)
#let highlight = strike.with(stroke: 10pt + rgb("abcdef88"), extent: 0.05em)

// Abuse thickness and transparency for redacting and highlighting stuff.
Sometimes, we work #redact[in secret].
There might be #highlight[redacted] things.
 underline()
