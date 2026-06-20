
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Differently styled spaces between text are not matched by regex rules.
// This is solely due to grouping rules, not space collapsing.
#show " ": "B"
#show "X": "B"
A C \
A#text(red)[ ]C \
A#text(red)[X]C