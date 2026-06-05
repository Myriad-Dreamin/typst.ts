
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In this bug stroke settings did not apply to math content.
// We expect all of these to have a green stroke.
#set text(stroke: green + 0.5pt)

A $B^2$ $ grave(C)' $