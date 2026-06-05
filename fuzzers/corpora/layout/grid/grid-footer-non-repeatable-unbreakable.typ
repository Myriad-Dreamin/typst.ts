
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 8em, width: auto)
#table(
  [h],
  table.footer(
    [a],
    [b],
    [c],
    repeat: false,
  )
)