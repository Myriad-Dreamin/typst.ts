
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The inner rectangle should not be yellow here.
A #box(rect(fill: yellow, inset: 5pt, rect())) B