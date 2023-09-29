// https://github.com/skyzh/typst-cv-template/blob/master/cv.typ

#show heading: set text(font: "Linux Biolinum")

#let style_color = rgb("#ffffff")

#set text(fill: style_color)
#show link: it => underline(stroke: style_color, it)
#set page(
 margin: (x: 0.9cm, y: 1.3cm),
//  fill: rgb("#343541")
)
#set par(justify: true)

#let chiline() = {v(-3pt); line(length: 100%, stroke: style_color); v(-5pt)}

= Alex Chi

skyzh\@cmu.edu |
#link("https://github.com/skyzh")[github.com/skyzh] | #link("https://skyzh.dev")[skyzh.dev]

== Education
#chiline()

#let education = [
  *#lorem(2)* #h(1fr) 2333/23 -- 2333/23 \
  #lorem(5) #h(1fr) #lorem(2) \
  - #lorem(10)
]

#education
#education
#education
#education

== Work Experience
#chiline()

#let experience = [
  *#lorem(2)* #h(1fr) 2333/23 -- 2333/23 \
  #lorem(5) #h(1fr) #lorem(2) \
  - #lorem(20)
  - #lorem(30)
  - #lorem(40)
]

#experience
#experience
#experience

== Projects
#chiline()

#let project = experience

#project
#project
#project
#project
#project
#project
