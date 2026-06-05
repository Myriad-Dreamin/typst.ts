
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test comparing durations
#test(duration(minutes: 20) > duration(minutes: 10), true)
#test(duration(minutes: 20) >= duration(minutes: 10), true)
#test(duration(minutes: 10) < duration(minutes: 20), true)
#test(duration(minutes: 10) <= duration(minutes: 20), true)
#test(duration(minutes: 10) == duration(minutes: 10), true)
#test(duration(minutes: 10) != duration(minutes: 20), true)
#test(duration(minutes: 10) <= duration(minutes: 10), true)
#test(duration(minutes: 10) >= duration(minutes: 10), true)
#test(duration(minutes: 20) < duration(minutes: 10), false)
#test(duration(minutes: 20) <= duration(minutes: 10), false)
#test(duration(minutes: 20) == duration(minutes: 10), false)