
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `trim` method; the pattern is not provided.
#let str = "Typst, LaTeX, Word, InDesign"
#let array = ("Typst", "LaTeX", "Word", "InDesign")
#test(str.split(",").map(s => s.trim()), array)
#test("".trim(), "")
#test("  ".trim(), "")
#test("\t".trim(), "")
#test("\n".trim(), "")
#test("\t \n".trim(), "")
#test(" abc ".trim(at: start), "abc ")
#test("\tabc ".trim(at: start), "abc ")
#test("abc\n".trim(at: end), "abc")
#test(" abc ".trim(at: end, repeat: true), " abc")
#test("  abc".trim(at: start, repeat: false), "abc")