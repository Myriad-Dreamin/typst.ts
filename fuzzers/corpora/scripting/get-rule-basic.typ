
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test basic get rule.
#context test(text.lang, "en")
#set text(lang: "de")
#context test(text.lang, "de")
#text(lang: "es", context test(text.lang, "es"))