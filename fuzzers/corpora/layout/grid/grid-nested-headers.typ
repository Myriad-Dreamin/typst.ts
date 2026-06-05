
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 12em)
#table(
  table.header(
    table(
      table.header(
        [b]
      ),
      [c],
      [d]
    )
  ),
  [a\ b]
)