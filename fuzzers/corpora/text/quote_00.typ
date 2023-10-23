
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Text direction affects author positioning
And I quote: #quote(attribution: [René Descartes])[cogito, ergo sum].

#set text(lang: "ar")
#quote(attribution: [عالم])[مرحبًا]
