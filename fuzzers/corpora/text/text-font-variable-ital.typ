
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set text(font: "Mona Sans")
Hello _Hello_

#text(variations: (ital: 0))[Hello]
#text(variations: (ital: 1))[Hello]