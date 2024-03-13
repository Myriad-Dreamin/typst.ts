#show quote.where(block: true): it => {

    align(center)[
      #rect(
        radius: 2mm,
        inset: 5mm,
        width: 100%,
        fill: gradient.linear(luma(240), luma(210)),
      )[
        #place(top + left, dy: -7mm)[
          #rect(
            radius: 1mm,
            width: 7mm,
            height: 5mm,
            fill: luma(120),
          )[#text(size: 20pt, baseline: -1.3pt, fill: white, font: "Trebuchet MS")[”]]
        ]
        #v(2mm)
        #align(left)[#emph(it.body)]
        #if it.has("attribution") and it.attribution != none {
          align(right)[-- #it.attribution]
        }
      ]
    ]
  }  

#quote(attribution: "some latin dude, probably", block: true)[#lorem(50)]