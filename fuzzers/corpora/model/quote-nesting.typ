
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test quote selection.
#set page(width: auto)
#set text(lang: "en")
=== EN
#quote[An apostroph'] \
#quote[A #quote[nested] quote] \
#quote[A #quote[very #quote[nested]] quote]

#set text(lang: "de")
=== DE
#quote[Satz mit Apostroph'] \
#quote[Satz mit #quote[Zitat]] \
#quote[A #quote[very #quote[nested]] quote]

#set smartquote(alternative: true)
=== DE Alternative
#quote[Satz mit Apostroph'] \
#quote[Satz mit #quote[Zitat]] \
#quote[A #quote[very #quote[nested]] quote]