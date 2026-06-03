
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test adding and subtracting durations.
#test(duration(weeks: 1, hours: 1), duration(weeks: 1) + duration(hours: 1))
#test(duration(weeks: 1, hours: -1), duration(weeks: 1) - duration(hours: 1))
#test(duration(days: 6, hours: 23), duration(weeks: 1) - duration(hours: 1))