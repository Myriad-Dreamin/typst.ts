
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test displaying of dates.
#test(datetime(year: 2023, month: 4, day: 29).display(), "2023-04-29")
#test(datetime(year: 2023, month: 4, day: 29).display("[year]"), "2023")
#test(
  datetime(year: 2023, month: 4, day: 29)
    .display("[year repr:last_two]"),
  "23",
)
#test(
  datetime(year: 2023, month: 4, day: 29)
    .display("[year] [month repr:long] [day] [week_number] [weekday]"),
  "2023 April 29 17 Saturday",
)

// Test displaying of times
#test(datetime(hour: 14, minute: 26, second: 50).display(), "14:26:50")
#test(datetime(hour: 14, minute: 26, second: 50).display("[hour]"), "14")
#test(
  datetime(hour: 14, minute: 26, second: 50)
    .display("[hour repr:12 padding:none]"),
  "2",
)
#test(
  datetime(hour: 14, minute: 26, second: 50)
    .display("[hour], [minute], [second]"), "14, 26, 50",
)

// Test displaying of datetimes
#test(
  datetime(year: 2023, month: 4, day: 29, hour: 14, minute: 26, second: 50).display(),
  "2023-04-29 14:26:50",
)

// Test getting the year/month/day etc. of a datetime
#let d = datetime(year: 2023, month: 4, day: 29, hour: 14, minute: 26, second: 50)
#test(d.year(), 2023)
#test(d.month(), 4)
#test(d.weekday(), 6)
#test(d.day(), 29)
#test(d.hour(), 14)
#test(d.minute(), 26)
#test(d.second(), 50)

#let e = datetime(year: 2023, month: 4, day: 29)
#test(e.hour(), none)
#test(e.minute(), none)
#test(e.second(), none)

// Test today
#test(datetime.today().display(), "1970-01-01")
#test(datetime.today(offset: auto).display(), "1970-01-01")
#test(datetime.today(offset: 2).display(), "1970-01-01")