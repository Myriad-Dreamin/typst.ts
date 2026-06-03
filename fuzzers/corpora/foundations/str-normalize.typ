
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test the `normalize` method.
#test("e\u{0301}".normalize(form: "nfc"), "é")
#test("é".normalize(form: "nfd"), "e\u{0301}")
#test("ſ\u{0301}".normalize(form: "nfkc"), "ś")
#test("ſ\u{0301}".normalize(form: "nfkd"), "s\u{0301}")