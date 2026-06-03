
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `split` method.
#test("abc".split(""), ("", "a", "b", "c", ""))
#test("abc".split("b"), ("a", "c"))
#test("a123c".split(regex("\d")), ("a", "", "", "c"))
#test("a123c".split(regex("\d+")), ("a", "c"))