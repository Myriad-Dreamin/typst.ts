
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(paper: "a10")

#let st = state("st", 0)

#let fn() = {
  st.update(i => i + 1)
  lorem(11)
  st.update(i => i - 1)
}

#grid(fn())

#fn()

Result: #context st.get()