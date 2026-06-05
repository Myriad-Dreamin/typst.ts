
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let envelope = symbol(
  "🖂",
  ("stamped", "🖃"),
  ("stamped.pen", "🖆"),
  ("lightning", "🖄"),
  ("fly", "🖅"),
)
#let one = symbol(
  "1",
  ("emoji", "1️")
)

#envelope
#envelope.stamped
#envelope.pen
#envelope.stamped.pen
#envelope.lightning
#envelope.fly
#one
#one.emoji