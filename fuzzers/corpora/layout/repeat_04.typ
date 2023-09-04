
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test single repeat in both directions.
A#box(width: 1fr, repeat(rect(width: 6em, height: 0.7em)))B

#set align(center)
A#box(width: 1fr, repeat(rect(width: 6em, height: 0.7em)))B

#set text(dir: rtl)
ريجين#box(width: 1fr, repeat(rect(width: 4em, height: 0.7em)))سون
