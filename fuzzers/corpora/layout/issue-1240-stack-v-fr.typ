
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 60pt)
#stack(
  dir: ltr,
  spacing: 1fr,
  stack([a], 1fr, [b]),
  stack([a], v(1fr), [b]),
)