
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 200pt)
#set text(8pt)

#let ls = state("lorem", lorem(30).split(" "))
#let loremum(count) = {
  context ls.get().slice(0, count).join(".").trim() + "."
  ls.update(list => list.slice(count))
}

#let fs = state("fader", red)
#let trait(title) = block[
  #context text(fill: fs.get())[
    *#title:* #loremum(1)
  ]
  #fs.update(color => color.lighten(30%))
]

#trait[Boldness]
#trait[Adventure]
#trait[Fear]
#trait[Anger]