
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test nested stretch calls.
$ stretch(=, size: #2em) \
  stretch(stretch(=, size: #4em), size: #50%) $

#let base = math.stretch($=$, size: 4em)
$ stretch(base, size: #50%) $

#let base = $stretch(=, size: #4em) $
$ stretch(base, size: #50%) $