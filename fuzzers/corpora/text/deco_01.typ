
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let redact = strike.with(stroke: 10pt, extent: 0.05em)
#let highlight-custom = strike.with(stroke: 10pt + rgb("abcdef88"), extent: 0.05em)

// Abuse thickness and transparency for redacting and highlighting stuff.
Sometimes, we work #redact[in secret].
There might be #highlight-custom[redacted] things.
