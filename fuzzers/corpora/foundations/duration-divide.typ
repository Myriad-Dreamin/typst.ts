
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test dividing durations with durations
#test(duration(minutes: 20) / duration(hours: 1), 1 / 3)
#test(duration(minutes: 20) / duration(minutes: 10), 2)
#test(duration(minutes: 20) / duration(minutes: 8), 2.5)