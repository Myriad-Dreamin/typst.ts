
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test that the language passed to the shaper has an effect.
#set text(font: "Ubuntu")

// Some lowercase letters are different in Serbian Cyrillic compared to other
// Cyrillic languages. Since there is only one set of Unicode codepoints for
// Cyrillic, these can only be seen when setting the language to Serbian and
// selecting one of the few fonts that support these letterforms.
Бб
#text(lang: "uk")[Бб]
#text(lang: "sr")[Бб]
