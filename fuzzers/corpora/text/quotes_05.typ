
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test changing properties within text.
"She suddenly started speaking french: #text(lang: "fr")['Je suis une banane.']" Roman told me.

Some people's thought on this would be #[#set smartquote(enabled: false); "strange."]
