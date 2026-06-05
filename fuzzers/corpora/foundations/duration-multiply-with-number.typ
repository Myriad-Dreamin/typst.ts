
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test multiplying and dividing durations with numbers.
#test(duration(minutes: 10) * 6, duration(hours: 1))
#test(duration(minutes: 10) * 2, duration(minutes: 20))
#test(duration(minutes: 10) * 2.5, duration(minutes: 25))
#test(duration(minutes: 10) / 2, duration(minutes: 5))
#test(duration(minutes: 10) / 2.5, duration(minutes: 4))