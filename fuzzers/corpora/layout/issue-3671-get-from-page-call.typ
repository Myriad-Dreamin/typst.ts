
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(margin: 5pt)
#context test(page.margin, 5pt)
#page(margin: 10pt, context test(page.margin, 10pt))