
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Ensure that there's no unconditional break at the end of a link.
#set page(width: 180pt, height: auto, margin: auto)
#set text(11pt)

For info see #link("https://myhost.tld").
