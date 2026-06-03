
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test vertical stretch interactions with attachments.
$arrow.t$
$stretch(arrow.t)^"map"$
$stretch(arrow.t, size: #2em)^"map"$
$stretch(arrow.t, size: #200%)^"map"$