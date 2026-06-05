
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `match` method.
#test("Is there a".match("for this?"), none)
#test(
  "The time of my life.".match(regex("[mit]+e")),
  (start: 4, end: 8, text: "time", captures: ()),
)