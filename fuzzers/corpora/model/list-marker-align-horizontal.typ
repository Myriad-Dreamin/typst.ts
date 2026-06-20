
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set list(marker: {
  counter("list").update(n => calc.max(n * 10, 1))
  context counter("list").display()
})

- Item
- Item
- Item

#set list(marker-align: start)
#counter("list").update(0)

- Item
- Item
- Item