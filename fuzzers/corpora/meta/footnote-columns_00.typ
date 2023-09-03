
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 120pt)
#align(center, strong[Title])
#show: columns.with(2)
#lorem(3) #footnote(lorem(6))
Hello there #footnote(lorem(2))
