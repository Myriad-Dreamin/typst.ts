
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(height: 70pt)
#table(
  rows: 16pt,
  ..range(6).map(str).flatten(),
)
