
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The word breaker naturally breaks arco-da-velha at arco-/-da-velha,
// so we shall repeat the hyphen, even that hyphenate is set to false.
#set page(width: 4cm)
#set text(lang: "pt")

Alguma coisa no arco-da-velha é algo que está muito longe.