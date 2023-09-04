
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#set page(width: 200pt)
#set text(8pt)

#let ls = state("lorem", lorem(1000).split("."))
#let loremum(count) = {
  ls.display(list => list.slice(0, count).join(".").trim() + ".")
  ls.update(list => list.slice(count))
}

#let fs = state("fader", red)
#let trait(title) = block[
  #fs.display(color => text(fill: color)[
    *#title:* #loremum(1)
  ])
  #fs.update(color => color.lighten(30%))
]

#trait[Boldness]
#trait[Adventure]
#trait[Fear]
#trait[Anger]
