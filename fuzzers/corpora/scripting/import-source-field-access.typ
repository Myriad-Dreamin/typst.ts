
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Usual importing syntax also works for function scopes
#let d = (e: enum)
#import d.e
#import d.e as renamed
#import d.e: item
#item(2)[a]