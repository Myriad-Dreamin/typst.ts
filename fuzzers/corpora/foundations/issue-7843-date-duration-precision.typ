
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test(
  datetime(year: 2030, month: 5, day: 25,            hour:  6, minute:  30, second:  19),
  datetime(year: 2030, month: 5, day: 25) + duration(hours: 6, minutes: 30, seconds: 19),
)
#test(
  datetime(year: 2030, month: 5, day: 25),
  datetime(year: 2030, month: 5, day: 26) - duration(hours: 24),
)

#let some-day = datetime(year: 2000, month: 6, day: 5)
#let two-hours = duration(hours: 2)
#test(some-day + 100 * two-hours, range(100).fold(some-day, (d, _) => d + two-hours))
#test(some-day - 100 * two-hours, range(100).fold(some-day, (d, _) => d - two-hours))