
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Values retrieved from function are not resolved at the moment.
// Ideally the left size would match the right size.
#let size = context [#text.size.to-absolute() #1em.to-absolute()]
$ size x^size x^x^size $