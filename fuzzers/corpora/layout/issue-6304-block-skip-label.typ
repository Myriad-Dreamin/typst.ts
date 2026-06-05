
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Ensure that labeling is skipped for an empty orphan frame.
#set page(height: 60pt)
A
#block(sticky: true)[B]
#block[C] <label>