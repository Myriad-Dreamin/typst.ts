
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// More tests for hyphenation of non-words.
#set text(hyphenate: true)
#block(width: 0pt, "doesn't")
#block(width: 0pt, "(OneNote)")
#block(width: 0pt, "(present)")

#set text(lang: "de")
#block(width: 0pt, "(bzw.)")