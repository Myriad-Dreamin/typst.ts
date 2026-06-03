
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#test("🏳️‍🌈!".codepoints(), ("🏳", "\u{fe0f}", "\u{200d}", "🌈", "!"))