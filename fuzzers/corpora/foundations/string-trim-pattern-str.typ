
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `trim` method; the pattern is a string.
#test("aabcaa".trim("a", repeat: false), "abca")
#test("aabca".trim("a", at: start), "bca")
#test("aabcaa".trim("a", at: end, repeat: false), "aabca")
#test(" abc\n".trim("\n"), " abc")
#test("whole".trim("whole", at: start), "")