
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let url = "https://typst.org/"
#let body = [Hello #box(width: 1fr, repeat[.])]

Inline: #link(url, body)

#link(url, block(inset: 4pt, [Block: ] + body))