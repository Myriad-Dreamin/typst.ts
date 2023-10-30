#import "/contrib/typst/diagram.typ": node, arr, commutative_diagram
#import "/docs/cookery/templates/page.typ": main-color, background-color, is-light-theme
#import "@preview/cetz:0.1.2"

#let forest = rgb("43a127")
    
/// utility functions
#let vadd(x,y) = (x.at(0) + y.at(0), x.at(1) + y.at(1))
#let vdiv(x,y) = (x.at(0) / y, x.at(1) / y)
#let vneg(x) = (-x.at(0), -x.at(1))

#let data-flow-graph(
  stroke: main-color,
  bg-color: background-color,
  light-theme: is-light-theme,
) = {
  let arr = arr.with(stroke: stroke)

  let adj-green = if light-theme {
    forest.darken(10%)
  } else {
    green
  }

  commutative_diagram(
    node_padding: (70pt, 50pt),
    bg-color: bg-color,
    node((0, 0), [
      Typst Documents
    ]),
    node((0, 2), [
      Preprocessed Artifact
    ]),
    node((1, 1), [
      #link("https://developer.mozilla.org/en-US/docs/Web/SVG")[SVG Document] ( `<svg/>` )
    ]),
    node((2, 1), [
      #link("https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas")[Canvas] ( `<canvas/>` )
    ]),
    arr((0, 0), (0, 2), [
      #set text(fill: adj-green)
      `precompile with theme and screen settings`
    ]),
    arr((0, 0), (1, 1), label_pos: 0.8em, {
      set text(fill: adj-green)
      rotate(17deg)[
        `compile to svg`
        #set text(fill: blue)
        #h(-0.5em) $space^dagger$
      ]
    }),
    arr((0, 0), (2, 1), label_pos: -0.6em, curve: -25deg, {
      set text(fill: blue)
      rotate(35deg)[`directly render` #h(-0.5em) $ space^(dagger dot.c dagger.double)$]
    }),
    arr((0, 2), (1, 1), label_pos: -0.8em, {
      set text(fill: blue)
      rotate(-17deg)[`render to svg` #h(-0.5em) $ space^dagger.double$]
    }),
    arr((1, 1), (2, 1), []),
    arr((0, 2), (2, 1), label_pos: 0.6em, curve: 25deg, {
      set text(fill: blue)
      rotate(-35deg)[`render to canvas` #h(-0.5em) $ space^(dagger.double)$]
    }),
  )
}


#let std-scale = scale
#let ir-feature-graph(
  stroke-color: main-color,
  bg-color: background-color,
  light-theme: is-light-theme,
) = {

  let stroke-factor = if light-theme {
    1
  } else {
    0.8
  }

  let adj-green = if light-theme {
    forest.darken(10%)
  } else {
    green
  }

  let data-fill = if light-theme {
    blue.lighten(60%)
  } else {
    blue.darken(10%)
  }

  let preview-img-fill = if light-theme {
    blue.lighten(80%)
  } else {
    blue.darken(20%)
  }

  let preview-img-fill2 = if light-theme {
    green.lighten(80%)
  } else {
    green.darken(20%)
  }

  let orange = if light-theme {
    yellow.mix(red.lighten(60%))
  } else {
    yellow.lighten(20%).mix(red.darken(20%))
  }

  let del-style = if light-theme {
    0.5pt + red.lighten(20%)
  } else {
    (0.5pt * stroke-factor) + red.lighten(20%)
  }

  // let typst-doc = text(font: "Garamond", [*doc*])
  let typst-doc = text(font: "Garamond", [*Typst doc*])

  cetz.canvas({
    import cetz.draw: *
    
    /// default style
    let light-doc-bg = black.lighten(98%)
    let dark-doc-bg = rgb("#343541")
    let rect = rect.with(stroke: (0.5pt*stroke-factor) + stroke-color)
    let circle = circle.with(stroke: (0.5pt*stroke-factor) + stroke-color)
    let line = line.with(stroke: (0.5pt*stroke-factor) + stroke-color)
    let light-line = line.with(stroke: (0.3pt*stroke-factor) + stroke-color)
    let exlight-line = line.with(stroke: (0.2pt*stroke-factor) + stroke-color)
    let preview-page = rect((0, 0), (0.8, 1), name: "preview-page", fill: if light-theme {
      light-doc-bg
    } else {
      dark-doc-bg
    })

    let x-diff = (3.5, 0)
    let y-diff = (0, 0.1)
    let rect-size = (3, 2)
    let data-sharing = (0, 0)
    let art-streaming = vadd(data-sharing, x-diff)
    let incr-rendering = vadd(art-streaming, x-diff)
    
    let prect(p) = rect(p, vadd(p, rect-size))
    // prect(data-sharing)
    // prect(art-streaming)
    // prect(incr-rendering)

    group({
      translate(data-sharing)
      content(vdiv((rect-size.at(0), 0), 2), std-scale(80%)[
        Data sharing
      ])
      translate(y-diff)

      let data-rect(q, c, name: "t") = {
        rect(q, vadd(q, (0.4, 0.4)), name: name, fill: data-fill)
        content(name, std-scale(80%, c))
      }

      let ref-rect(q, c) = {
        circle(vadd(q, (0.2, 0.2)), radius: 0.2, name: "t", fill: orange)
        content("t", std-scale(80%, c))
      }

      let sref-rect(q, name) = {
        let r = 0.08
        circle(vadd(q, (r, r)), radius: r, name: name, fill: orange, stroke: 0.2pt + stroke-color)
      }

      let q = (0.1, 1.5)
      data-rect(q, [T])
      let q = vadd(q, (0.4 + 0.1, 0))
      ref-rect(q, [&])

      let crect(s: none, fill: black, stroke: 0pt) = rect(vneg(vdiv(s, 2)), vdiv(s, 2), fill: fill, stroke: stroke)

      let br = crect.with(s: (3, 1), fill: light-doc-bg, stroke: black + 0.2pt)
      let sr = crect.with(s: (2.5, 2), fill: dark-doc-bg, stroke: stroke-color + 0.2pt)
      range(0, 7).zip(
        (("d", "data-d"), ("e", "data-e"), ("a", "data-a"),
        ("b", "data-b"), ("f", "data-f"), (br, "data-lr"), (sr, "data-dr"))
      ).map(((t, c)) => {
        let (c, tag) = c
        let q = (0.1 + t * 0.4, 0.1)
        data-rect(q, name: tag, if type(c) == str {
          c
        } else {
          ""
        })
        if type(c) != str {
          group({
            translate(vadd(q, (0.2, 0.2)))
            scale(1/10)
            c()
          })
        }
      }).sum()

      group({
        translate((0, 1.2))

        let q = (0.3, 0)
        rect(q, vadd(q, (0.66, 0.22)), fill: light-doc-bg, stroke: black + 0.2pt, name: "light-doc")
        content("light-doc", std-scale(40%, [
          #set text(fill: black)
          dead beef
        ]))
        sref-rect((0.1, 0), "light-doc")

        translate((0.29*6, 0))

        let q = (0.4, 0)
        let t = 0.33
        rect(q, vadd(q, (t/4*5, t)), fill: dark-doc-bg, stroke: stroke-color + 0.2pt, name: "dark-doc")
        content((v => vadd(v, (-0.05, 0)), "dark-doc"), std-scale(30%, [
          #set text(fill: stroke-color)
          dead #linebreak()
          beef
        ]))
        sref-rect((0.1, 0), "dark-doc")

        translate((-0.29*6, 0))

        let flow-stroke = 0.15pt + stroke-color

        translate((0, -0.6))
        range(0, 10).zip(("d", "e", "a", "d", "b", "e", "e", "f", "lr", "dr")).map(((t, c)) => {
          let q = (0.1 + t * 0.29, 0)
          let tag = "ref-l2-" + str(t)
          sref-rect(q, tag)
          line("ref-l2-" + str(t) + ".top", "data-" + c + ".top", stroke: flow-stroke)
        }).sum()
        let l2-line(t, tag: "root") = line(tag + ".top", "ref-l2-" + str(t) + ".bottom", stroke: flow-stroke)

        translate((0, 0.3))
        sref-rect((0.1, 0), "text-fff")
        content((0.57, 0.1-0.02), std-scale(40%, [fill=\#fff]))

        range(0, 8).map(l2-line.with(tag: "text-fff")).sum()

        line("light-doc.top", "text-fff.bottom", stroke: flow-stroke)

        group({
          translate((0.29*3, 0))

          sref-rect((0.1, 0), "text-000")
          content((0.57, 0.1-0.02), std-scale(40%, [fill=\#000]))

          range(0, 8).map(l2-line.with(tag: "text-000")).sum()

          line("dark-doc.top", "text-000.bottom", stroke: flow-stroke)
        })

        group({
          translate((0.29*6, 0))

          sref-rect((0.1, 0), "white-bg")

          l2-line(8, tag: "white-bg")

          line("light-doc.top", "white-bg.bottom", stroke: flow-stroke)

          translate((0.29*1, 0))

          sref-rect((0.1, 0), "dark-bg")

          l2-line(9, tag: "dark-bg")

          line("dark-doc.top", "dark-bg.bottom", stroke: flow-stroke)
        })
      })
      
    })

    group({
      translate(art-streaming)
      content(vdiv((rect-size.at(0), 0), 2), std-scale(80%)[
        Artifact Streaming
      ])
      translate(y-diff)
      // let q = (1.8, 1.5)
      // rect(q, vadd(q, (0.8, 0.4)), name: "typst-doc", stroke: 0pt)
      // content("typst-doc", std-scale(80%, typst-doc))

      let rect-r = 0.16
      let data-rect(q, c, name: "t") = {
        rect(q, vadd(q, (rect-r*2, rect-r*2)), name: name, fill: data-fill)
        content(name, std-scale(80%, c))
      }

      let ref-rect(q, c) = {
        circle(vadd(q, (rect-r, rect-r)), radius: rect-r, name: "t", fill: orange)
        content("t", std-scale(80%, c))
      }

      let bundle-rect(q, name: "t") = {
        let m = (0.05, 0.05)
        rect(q, vadd(q, (0.8, 0.4)), name: name)
        group({
          translate((0.03, 0.03))
          data-rect(vadd(q, (0.05*0.4, 0)), [])
          let q = vadd(q, (0.4 - 0.05*0.2, 0))
          ref-rect(q, [])
        })
      }

      let preview-content0 = {
        translate((0.05, 1-0.2))
        // title
        light-line((0.15, 0), (0.55, 0))
        translate((0, -0.08))
        exlight-line((0.05, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.4, 0))
      }

      let preview-content1 = {
        
        translate((0, -0.06))
        rect((0.05, 0), (0.65, -0.24), stroke: 0pt, fill: preview-img-fill, name: "picture")
        content("picture", std-scale(20%)[$lambda x = integral.double x dif x$])
        translate((0, -0.24))
      }

      translate((-0.05, 1.24))
      scale(0.9)
      bundle-rect((0.1, 0))
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        preview-content0
      })
      // text-decender = 0.02
      content((1.1, rect-r*2-0.02), std-scale(90%)[..])
      translate((1.1, 0))
      bundle-rect((0.1, 0))
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        
        preview-content0
        preview-content1
      })
      // text-decender = 0.02
      content((1.1, rect-r*2-0.02), std-scale(90%)[..])
      translate((1.1, 0))
      bundle-rect((0.1, 0))
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        
        preview-content0
        preview-content1

        translate((0, -0.03))
        exlight-line((0.05, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.6, 0))
      })
    })

    group({
      translate(incr-rendering)
      content(vdiv((rect-size.at(0), 0), 2), std-scale(80%)[
        Incremental Rendering
      ])
      translate(y-diff)
      // let q = (1.8, 1.5)
      // rect(q, vadd(q, (0.8, 0.4)), name: "typst-doc", stroke: 0pt)
      // content("typst-doc", std-scale(80%, typst-doc))

      let rect-r = 0.16
      let data-rect(q, c, name: "t") = {
        rect(q, vadd(q, (rect-r*2, rect-r*2)), name: name, fill: data-fill)
        content(name, std-scale(80%, c))
      }

      let ref-rect(q, c) = {
        circle(vadd(q, (rect-r, rect-r)), radius: rect-r, name: "t", fill: orange)
        content("t", std-scale(80%, c))
      }
      
      let dd-cross(data-off, data-end) = {
        line(data-off, data-end, stroke: del-style)
        let (x1, y1) = data-off
        let (x2, y2) = data-end
        line((x2, y1), (x1, y2), stroke: del-style)
      }

      let bundle-rect(q, name: "t", dd: false) = {
        let m = (0.05, 0.05)
        rect(q, vadd(q, (0.8, 0.4)), name: name)
        group({
          translate((0.03, 0.03))
          let data-off = vadd(q, (0.05*0.4, 0))
          let data-end = vadd(q, (rect-r*2, rect-r*2))
          data-rect(data-off, [])
          let ref-off = vadd(q, (0.4 - 0.05*0.2, 0))
          let ref-end = vadd(ref-off, (rect-r*2, rect-r*2))
          ref-rect(ref-off, [])
          if dd {
            dd-cross(data-off, data-end)
            let sqrt-rect-r = rect-r*(1-calc.sqrt(1/2))
            let srr = (sqrt-rect-r, sqrt-rect-r)
            dd-cross(vadd(srr, ref-off), vadd(vneg(srr), ref-end))
          }
        })
      }

      let preview-content(img-content, fill: none) = {
        translate((0.05, 1-0.2))
        // title
        light-line((0.15, 0), (0.55, 0))
        translate((0, -0.08))
        exlight-line((0.05, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.4, 0))
        
        translate((0, -0.06))
        if fill != none {
          rect((0.05, 0), (0.65, -0.24), stroke: 0pt, fill: fill, name: "picture")
          content("picture", std-scale(20%, img-content))
        }
        translate((0, -0.24))

        translate((0, -0.03))
        exlight-line((0.05, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.7, 0))
        translate((0, -0.03))
        exlight-line((0.00, 0), (0.6, 0))
      }

      translate((-0.05, 1.24))
      scale(0.9)
      bundle-rect((0.1, 0))
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        preview-content(fill: preview-img-fill)[$lambda x = integral.double x dif x$]
      })
      // text-decender = 0.02
      content((1.1, rect-r*2-0.02), std-scale(90%)[..])
      translate((1.1, 0))
      bundle-rect((0.1, 0), dd: true)
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        preview-content[]
      })
      // text-decender = 0.02
      content((1.1, rect-r*2-0.02), std-scale(90%)[..])
      translate((1.1, 0))
      bundle-rect((0.1, 0))
      group({
        translate((0.1, -1.1))
        preview-page
        // content("preview-page.top-left", anchor: "top-left", std-scale(7.5%,
        // origin: top + left, box(inset: 10pt, width: 280pt, height: auto,
        // clip: true, lorem(20))))
        
        
        preview-content(fill: preview-img-fill2)[$lambda = c x^2$]
      })
    })
  })
}
