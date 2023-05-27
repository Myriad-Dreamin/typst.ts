// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(title: "", authors: (), body) = {
  // Set the document's basic properties.
  set document(author: authors, title: title)
  set page(numbering: "1", number-align: center)
  set text(font: "New Computer Modern", lang: "en")
  show math.equation: set text(weight: 400)

  // Title row.
  align(center)[
    #block(text(weight: 700, 1.75em, title))
  ]

  // Author information.
  pad(
    top: 0.5em,
    bottom: 0.5em,
    x: 2em,
    grid(
      columns: (1fr,) * calc.min(3, authors.len()),
      gutter: 1em,
      ..authors.map(author => align(center, text(font: "New Computer Modern Sans", author))),
    ),
  )

  // Main body.
  set par(justify: true)

  body
}

#show: project.with(
  title: "Math Document",
)


#let long-symbol(sym, factor) = {
  assert(type(sym) == "symbol", message: "Input needs to be a symbol")
  assert(type(factor) == "integer" or type(factor) == "float", message: "Scale factor must be a number")
  assert(factor >= 1, message: "Scale factor must be >= 1")
  
  factor = 5*factor - 4
  let body = [#sym]
  style(styles => {
    let (body-w,body-h) = measure(body,styles).values()
    align(left)[
      #box(width: body-w*2/5,height: body-h,clip: true)[
        #align(left)[
          #body
        ]
      ]
      #h(0cm)
      #box(height: body-h, width: body-w*1/5*factor)[
        #scale(x: factor*100%,origin:left)[
          #box(height: body-h, width: body-w*1/5,clip:true)[
            #align(center)[
              #body
            ]
          ]
        ]
      ]
      #h(0cm)
      #box(width: body-w*2/5,clip: true)[
        #align(right)[
          #body
        ]
      ]
    ]
  })
}

$
  f: X attach(limits(#long-symbol(sym.arrow.r.hook,8)), t: "injective map") Y\
  #long-symbol(sym.arrow.l.r.double.long,10) \
  #long-symbol(sym.arrow.l.filled,5)
$