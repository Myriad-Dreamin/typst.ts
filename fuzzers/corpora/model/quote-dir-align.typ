
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Text direction affects block alignment
#set text(font: ("Libertinus Serif", "Noto Sans Arabic"))
#set quote(block: true)
#quote(attribution: [René Descartes])[cogito, ergo sum]

#set text(lang: "ar")
#quote(attribution: [عالم])[مرحبًا]