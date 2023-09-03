
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test hard line break (leads to two paragraphs in unicode-bidi).
#set text(lang: "ar", font: ("Noto Sans Arabic", "PT Sans"))
Life المطر هو الحياة \
الحياة تمطر is rain.
