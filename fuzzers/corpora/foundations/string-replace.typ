
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `replace` method with `Str` replacements.
#test("ABC".replace("", "-"), "-A-B-C-")
#test("Ok".replace("Ok", "Nope", count: 0), "Ok")
#test("to add?".replace("", "How ", count: 1), "How to add?")
#test("AB C DEF GH J".replace(" ", ",", count: 2), "AB,C,DEF GH J")
#test("Walcemo"
  .replace("o", "k")
  .replace("e", "o")
  .replace("k", "e")
  .replace("a", "e"),
  "Welcome"
)
#test("123".replace(regex("\d$"), "_"), "12_")
#test("123".replace(regex("\d{1,2}$"), "__"), "1__")