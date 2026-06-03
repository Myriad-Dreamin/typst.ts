
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Smart quotes were not appearing in the PDF outline, because they didn't
// implement `PlainText`.
= It's "Unnormal Heading"
= It’s “Normal Heading”

#set smartquote(enabled: false)
= It's "Unnormal Heading"
= It's 'single quotes'
= It’s “Normal Heading”