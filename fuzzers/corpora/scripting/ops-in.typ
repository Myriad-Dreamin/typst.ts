
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test `in` operator.
#test("hi" in "worship", true)
#test("hi" in ("we", "hi", "bye"), true)
#test("Hey" in "abHeyCd", true)
#test("Hey" in "abheyCd", false)
#test(5 in range(10), true)
#test(12 in range(10), false)
#test("" in (), false)
#test("key" in (key: "value"), true)
#test("value" in (key: "value"), false)
#test("Hey" not in "abheyCd", true)
#test("a" not
/* fun comment? */ in "abc", false)
#test("sys" in std, true)
#test("system" in std, false)