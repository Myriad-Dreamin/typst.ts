
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// In languages like French and German, manually added spaces before and after
// guillemets should not be breakable.
#set page(width: 125pt)

#set text(lang: "fr")
Les principales « guillemets ».\
Et les autres ‹ guillemets › en français & suisse romande.

#set text(lang: "de")
Alternative »Anführungszeichen« in DE & AT.