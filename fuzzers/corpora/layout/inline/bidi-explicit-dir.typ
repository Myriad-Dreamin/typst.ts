
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test explicit dir
#set text(dir: rtl)
#text("8:00 - 9:00", dir: ltr) בבוקר
#linebreak()
ב #text("12:00 - 13:00", dir: ltr) בצהריים