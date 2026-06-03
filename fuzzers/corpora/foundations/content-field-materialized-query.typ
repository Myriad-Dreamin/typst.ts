
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test it with query.
#set raw(lang: "rust")
#context query(<myraw>).first().lang
`raw` <myraw>