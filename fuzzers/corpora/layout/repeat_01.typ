
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test dots with RTL.
#set text(lang: "ar")
مقدمة #box(width: 1fr, repeat[.]) 15
