#let text_base_size=10pt
#let text_h1_size=20pt
#let header_size_increment=3pt
#set page(
  paper: "a4",
  header: align(right)[
    Multi-purpose Combat Chassis
  ],
  numbering: "1",
  margin: (x:20mm, y:12.7mm)
)

#set par(justify: true)
#set text(
  font: "LXGW WenKai",
  size: text_base_size,
  lang: "zh",
)
#let emp_block(body,fill:luma(230),stroke:orange) = {
  block(
    width:100%,
    fill: fill,
    inset: 8pt,
    radius: 0pt,
    stroke: (left:(stroke+3pt),right:(luma(200)+3pt)),
    body,
  )
}

#let booktab() = {
  block(
    width: 100%,
    [
      #line(length: 100%,stroke: 3pt+luma(140))
      #move(line(length: 100%), dx: 0pt, dy: -9pt)
    ]
  )
  v(0pt, weak: true)
}

#show heading: it => block[
  #let heading_size=text_h1_size - (it.level - 1) * header_size_increment
  #set text(size: heading_size, font: "HarmonyOS Sans SC")
  #emph(it.body)
]

#let img = "/assets/files/tiger.jpg"

= Seed

#outline(title:none, indent:auto, )

#booktab()

Seed2 Seed4

Seed3 Seed4

#pagebreak()
== #lorem(3)

#emp_block()[
 Seed4 Seed4 Seed4 Seed4
]
#booktab()

 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4 Seed4

#pagebreak()
== 隼的轻武器

#booktab()

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#let images_rkg3=("tiger.jpg","tiger.jpg")

#align(center)[
  #stack(
    dir: ltr,
    ..images_rkg3.map(n=>align(center)[#image(img, height:15%)])
  )
]

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#let images_pistol=("tiger.jpg","tiger.jpg")
#let cell = rect.with(
  inset: 8pt,
  fill: rgb("e4e5ea"),
  width: 100%,
  radius: 6pt
)
#grid(
  columns: (1fr,1fr),
  rows: (),
  gutter: 3pt,
  ..images_pistol.map(n=>align(center)[#image(img, width:20%)])
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#figure(
  image(img, width: 50%)
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#figure(
  image(img, width: 50%)
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

=== 电磁枪

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#pagebreak()
== 武器舱和背部重武器

#booktab()

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#figure(
  image(img, width: 60%),
  caption: [30mm Rapid Railgun with Extended Barrel, also retrofitted as 2nd stage rail on Arclight]
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#figure(
  image(img, width: 60%),
  caption: [TGLS/Tactical Graviton Laser(Lance) System]
)

#lorem(1000)

#figure(
  image(img, width: 60%),
  caption: [炮管展开60°的状态]
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#figure(
  image(img, width: 80%),
  caption: [4Sure Ballistics Man-Portable ASAT Missile]
)

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed

#pagebreak()
== 辅助机 —— "FRAMER"

#booktab()

Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed Seed
