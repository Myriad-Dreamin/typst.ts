
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test adding and subtracting durations and dates.
#let d = datetime(day: 1, month: 1, year: 2000)
#let d2 = datetime(day: 1, month: 2, year: 2000)
#test(d + duration(weeks: 2), datetime(day: 15, month: 1, year: 2000))
#test(d + duration(days: 3), datetime(day: 4, month: 1, year: 2000))
#test(d + duration(weeks: 1, days: 3), datetime(day: 11, month: 1, year: 2000))
#test(d2 + duration(days: -1), datetime(day: 31, month: 1, year: 2000))
#test(d2 + duration(days: -3), datetime(day: 29, month: 1, year: 2000))
#test(d2 + duration(weeks: -1), datetime(day: 25, month: 1, year: 2000))
#test(d + duration(days: -1), datetime(day: 31, month: 12, year: 1999))
#test(d + duration(weeks: 1, days: -7), datetime(day: 1, month: 1, year: 2000))
#test(d2 - duration(days: 1), datetime(day: 31, month: 1, year: 2000))
#test(d2 - duration(days: 3), datetime(day: 29, month: 1, year: 2000))
#test(d2 - duration(weeks: 1), datetime(day: 25, month: 1, year: 2000))
#test(d - duration(days: 1), datetime(day: 31, month: 12, year: 1999))
#test(datetime(day: 31, month: 1, year: 2000) + duration(days: 1), d2)
#test(
  datetime(day: 31, month: 12, year: 2000) + duration(days: 1),
  datetime(day: 1, month: 1, year: 2001),
)