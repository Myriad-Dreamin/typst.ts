
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test font switch.
#let here = text.with(font: "Noto Sans")
$#here[f] := #here[Hi there]$.
