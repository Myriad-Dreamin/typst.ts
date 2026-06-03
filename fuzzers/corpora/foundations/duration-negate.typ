
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test negating durations.
#test(-duration(hours: 2), duration(hours: -2))