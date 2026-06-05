
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Nested math content has styles overwritten by the inner equation.
// Ideally the widths would match the actual length of the arrows.
#let arrow = $stretch(->)^"much text"$
$ arrow A^arrow A^A^arrow $
#let width = context measure(arrow).width
$ width A^width A^A^width $