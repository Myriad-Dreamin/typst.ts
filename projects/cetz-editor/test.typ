
#import "@preview/cetz:0.1.2"
#set page(margin: 1pt, width: 602pt, height: 602pt)
#let debug-label(_) = ()
#cetz.canvas({
import cetz.draw: *
  let x-circle(rad: 200, inner-text: "", tag: black, node-label: none) = {
  circle((0, 0), radius: rad, name: node-label)
  debug-label((-rad*0.7, -rad*0.7))
  content(node-label, inner-text)
}
  let x-rect(x: 200, y: none, inner-text: "", tag: black, node-label: none) = {
  let y = if y == none {
    x
  } else {
    y
  }
  rect((0, 0), (x, y), name: node-label)
  debug-label((0, 0))
  content(node-label, inner-text)

}
  let x-arrow(start: (0, 10), end: (50, 10), inner-text: "", mark: (end: ">"), tag: black, node-label: none) = {
  set-style(mark: (fill: none, size: 14))
  line(start, end, name: "t", mark: mark)
  content("t", inner-text)
}
  translate((0, 0))
  x-circle(node-label: "c0", rad: 50) // c0
  translate((50, 50))
  x-rect(node-label: "c1", x: 100) // c1
  translate((150, -50))
  x-circle(node-label: "c2", rad: 50) // c2
  translate((-200, 0))
  x-arrow(node-label: "c1toc2", start: "c1", end: "c2") // c1toc2
}, length: 1pt)