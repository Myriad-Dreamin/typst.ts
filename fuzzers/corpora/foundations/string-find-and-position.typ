
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `find` and `position` methods.
#let date = regex("\d{2}:\d{2}")
#test("Hello World".find("World"), "World")
#test("Hello World".position("World"), 6)
#test("It's 12:13 now".find(date), "12:13")
#test("It's 12:13 now".position(date), 5)