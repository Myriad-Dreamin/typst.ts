
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `trim` method; the pattern is a regex.
#test("".trim(regex(".")), "")
#test("123abc456".trim(regex("\d")), "abc")
#test("123abc456".trim(regex("\d"), repeat: false), "23abc45")
#test("123a4b5c678".trim(regex("\d"), repeat: true), "a4b5c")
#test("123a4b5c678".trim(regex("\d"), repeat: false), "23a4b5c67")
#test("123abc456".trim(regex("\d"), at: start), "abc456")
#test("123abc456".trim(regex("\d"), at: end), "123abc")
#test("123abc456".trim(regex("\d+"), at: end, repeat: false), "123abc")
#test("123abc456".trim(regex("\d{1,2}$"), repeat: false), "123abc4")
#test("hello world".trim(regex(".")), "")
#test("12306".trim(regex("\d"), at: start), "")
#test("12306abc".trim(regex("\d"), at: start), "abc")
#test("whole".trim(regex("whole"), at: start), "")
#test("12306".trim(regex("\d"), at: end), "")
#test("abc12306".trim(regex("\d"), at: end), "abc")
#test("whole".trim(regex("whole"), at: end), "")