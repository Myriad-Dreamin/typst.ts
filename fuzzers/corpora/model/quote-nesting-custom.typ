
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// With custom quotes.
#set smartquote(quotes: (single: ("<", ">"), double: ("(", ")")))
#quote[A #quote[nested] quote]