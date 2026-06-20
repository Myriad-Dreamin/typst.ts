
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test("Typst".ends-with("st"), true)
#test("Typst".ends-with(regex("\\d*")), true)
#test("Typst".ends-with(regex("\\d+")), false)
#test("Typ12".ends-with(regex("\\d+")), true)
#test("typst13".ends-with(regex("1[0-9]")), true)
#test("typst113".ends-with(regex("1[0-9]")), true)
#test("typst23".ends-with(regex("1[0-9]")), false)