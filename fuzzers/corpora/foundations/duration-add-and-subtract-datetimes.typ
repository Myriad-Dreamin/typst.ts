
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test adding and subtracting durations and datetimes.
#test(
  datetime(day: 1, month: 1, year: 2000, hour: 12, minute: 0, second: 0)
    + duration(weeks: 1, days: 3, hours: -13, minutes: 10, seconds: -10 ),
  datetime(day: 10, month: 1, year: 2000, hour: 23, minute: 9, second: 50),
)
#test(
  datetime(day: 1, month: 1, year: 2000, hour: 12, minute: 0, second: 0)
    + duration(weeks: 1, days: 3, minutes: 10)
    - duration(hours: 13, seconds: 10),
  datetime(day: 10, month: 1, year: 2000, hour: 23, minute: 9, second: 50),
)