
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test start and end alignment.
#rotate(-30deg, origin: end + horizon)[Hello]

#set text(lang: "de")
#align(start)[Start]
#align(end)[Ende]

#set text(lang: "ar")
#align(start)[يبدأ]
#align(end)[نهاية]
