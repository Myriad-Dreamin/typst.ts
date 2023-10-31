
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Use language quotes for missing keys, allow partial reset
#set smartquote(quotes: "«»")
"Double and 'Single' Quotes"

#set smartquote(quotes: (double: auto, single: "«»"))
"Double and 'Single' Quotes"
