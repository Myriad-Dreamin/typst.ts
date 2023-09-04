
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test word spacing relative to the font's space width.
#set text(spacing: 50% + 1pt)
This is tight.
