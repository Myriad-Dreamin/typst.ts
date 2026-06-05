
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test font switch.
// Warning: 29-40 unknown font family: noto sans
#let here = text.with(font: "Noto Sans")
$#here[f] := #here[Hi there]$.