#let node(pos, label, ..opts) = (
  kind: "node",
  pos: pos,
  label: label,
  id: repr(opts.pos().at(0, default: label)),
)

#let arr(start, end, label,
         start-space: none, end-space: none,
         label-pos: left, curve: 0deg, stroke: 0.05em, ..options) = {
	//Check that options are compatible
	if ("nat" in options.pos() or "two" in options.pos()){
		if("inj" in options.pos()){
			panic("Hooked double-stemmed arrows are not supported")
		}
		if("surj" in options.pos()){
			panic("Surjective double-stemmed arrows are not supported")
		}
		if("def" in options.pos()){
			panic("Assignment double-stemmed arrows are not supported")
		}
	}
	if("two" in options.pos() and "nat" in options.pos()){
		panic("Two and Natural are incompatible stem options")
	}
	if("inj" in options.pos()){
		if("def" in options.pos()){
			panic("Arrow should begin in incompatible ways")
		}
		if("bij" in options.pos()){
			panic("Arrow should begin in incompatible ways")
		}
	}
	if("def" in options.pos()){
		if("bij" in options.pos()){
			panic("Arrow should begin in incompatible ways")
		}
	}
  (
    kind: "arrow",
    start: start,
    end: end,
    label: label,
    start-space: start-space,
    end-space: end-space,
    label-pos: label-pos,
    curve: curve,
    stroke: stroke,
    options: options.pos(),
  )
}

#let commutative-diagram(
  node-padding: (70pt, 70pt),
  arr-clearance: 0.7em,
  padding: 1.5em,
  debug: false,
  ..entities
) = {
  context {

    // useful utility function
    let _center(c) = {
      let m = measure(c)
      place(dx: -0.5*m.width, dy: -0.5*m.height, c)
    }
  
    // get nodes and arrows from the function arguments
    let entities = entities.pos()
    let nodes = entities.filter(e => e.kind == "node")
    let myarrows = entities.filter(e => e.kind == "arrow")
    let pos-from-id = (:)
  
    // adjust node and arrow coordinates so that the min is (0, 0)
    let min-row = calc.min(..nodes.map(node => node.pos.at(0)))
    let min-col = calc.min(..nodes.map(node => node.pos.at(1)))
    let max-row = calc.max(..nodes.map(node => node.pos.at(0)))
    let max-col = calc.max(..nodes.map(node => node.pos.at(1)))
  
    for i in range(nodes.len()) {
      nodes.at(i).pos.at(0) -= min-row
      nodes.at(i).pos.at(1) -= min-col
      pos-from-id.insert(nodes.at(i).id, nodes.at(i).pos)
    }
  
    myarrows = myarrows.map(arr => {
      if type(arr.start) == array {
        arr.start.at(0) -= min-row
        arr.start.at(1) -= min-col
      } else if repr(arr.start) in pos-from-id {
        arr.start = pos-from-id.at(repr(arr.start), default: none)
      } else {
        panic("Error while processing arrow from " +
              repr(arr.start) + " to " + repr(arr.end) +
              ": can't find a node with id = " + repr(arr.start))
      }
      if type(arr.end) == array {
        arr.end.at(0) -= min-row
        arr.end.at(1) -= min-col
      } else if repr(arr.end) in pos-from-id {
        arr.end = pos-from-id.at(repr(arr.end), default: none)
      } else {
        panic("Error while processing arrow from " +
              repr(arr.start) + " to " + repr(arr.end) +
              ": can't find a node with id = " + repr(arr.end))
      }
      arr
    })
  
    max-row -= min-row
    max-col -= min-col
    min-col = 0
    min-row = 0
  
    // measure the size of the various rows and columns
    let col-sizes = ()
    let row-sizes = ()
    for r in range(0, max-row+1) {
      row-sizes.push(0pt)
    }
    for c in range(0, max-col+1) {
      col-sizes.push(0pt)
    }
    for node in nodes {
      let m = measure(node.label)
      if m.width > col-sizes.at(node.pos.at(1)) {
        col-sizes.at(node.pos.at(1)) = m.width
      }
      if m.height > row-sizes.at(node.pos.at(0)) {
        row-sizes.at(node.pos.at(0)) = m.height
      }
    }
    let row-pos = row-sizes
    let col-pos = col-sizes
    row-pos.at(0) /= 2
    col-pos.at(0) /= 2
    for r in range(1, max-row+1) {
      //row-pos.at(r) += row-pos.at(r - 1) + node-padding.at(1)
	row-pos.at(r) = row-pos.at(r - 1) + row-sizes.at(r - 1)/2 + row-sizes.at(r)/2 + node-padding.at(1)
    }
    for c in range(1, max-col+1) {
      //col-pos.at(c) += col-pos.at(c - 1) + node-padding.at(0)
	col-pos.at(c) = col-pos.at(c - 1) + col-sizes.at(c - 1)/2 + col-sizes.at(c)/2 + node-padding.at(0)
    }
  
    // total diagram dimensions
    let height = row-sizes.fold(-node-padding.at(1), (x, y) =>
                                x+y+node-padding.at(1))
    let width = col-sizes.fold(-node-padding.at(0), (x, y) =>
                                x+y+node-padding.at(0))

    // useful functions used many times later
    let coords(pos) = (
      col-pos.at(pos.at(1)), row-pos.at(pos.at(0))
    )
  
    let size-at(pos) = {
      for node in nodes {
        if node.pos == pos {
          return measure(node.label)
        }
      }
      return (width: 0pt, height: 0pt)
    }
  
    // units conversion
    let pt-per-em = measure(rect(width: 1em)).width
    let to-abs(l) = l.abs + l.em * pt-per-em

    // various vector utility functions
    let v-add(a, b) = (a.at(0) + b.at(0), a.at(1) + b.at(1))
    let v-sub(a, b) = (a.at(0) - b.at(0), a.at(1) - b.at(1))
    let v-mul(a, x) = (a.at(0) * x, a.at(1) * x)
    let v-dir(a, l) = v-mul((calc.cos(a), calc.sin(a)), l)
    let v-add-dir(p, a, l) = v-add(p, v-dir(a, l))
    let v-length(vv) = calc.sqrt(calc.pow(to-abs(vv.at(0))/1mm, 2) +
                                 calc.pow(to-abs(vv.at(1))/1mm, 2))*1mm
  
    // measures the length of a segment starting in the center
    // of a rectangle and ending on one of the rectangle's edges,
    // given the angle of the segment
    let measure-at-angle(m, a, padding) = {
      if calc.sin(a) == 0 {
        m.width / 2 + padding
      } else if calc.cos(a) == 0 {
        m.height / 2 + padding
      } else {
        calc.min(
          (m.width / 2 + padding) / calc.abs(calc.cos(a)), 
          (m.height / 2 + padding) / calc.abs(calc.sin(a)),
          calc.max(m.width, m.height) / 2 + padding,
        )
      }
    }
  
    // the box where all the diagram's elements are
    box(
      width: width + 2*padding, height: height + 2*padding,
      stroke: if debug { 0.5pt + red } else { none },
      inset: padding,
      {

      for arr in myarrows {
        let start = coords(arr.start)
        let end = coords(arr.end)
        // arrow angle measured clockwise from the right
        // i know that clockwise is disgusting, but rotate rotates clockwise
        let angle = calc.atan2((end.at(0) - start.at(0)) / 1pt,
                               (end.at(1) - start.at(1)) / 1pt)
        let curve-angle = arr.curve
        let start-space = arr.start-space
        let end-space = arr.end-space
        if start-space == none {
          start-space = measure-at-angle(size-at(arr.start),
                                         angle - curve-angle,
                                         to-abs(arr-clearance))
        }
        if end-space == none {
          end-space = measure-at-angle(size-at(arr.end),
                                       angle + curve-angle,
                                       to-abs(arr-clearance))
        }
        let astart = v-add-dir(start, angle - curve-angle, start-space)
        let aend = v-add-dir(end, angle + 180deg + curve-angle, end-space)
      
        // draw the arrow's tip
        place(dx: aend.at(0), dy: aend.at(1),
          rotate(angle + curve-angle + 90deg, origin: top+left,
            if "surj" in arr.options {
              move(dy: 0.21em, _center(
                box(clip:true, height: 0.42em, $arrow.t.twohead$)
              ))
              aend = v-add-dir(aend, angle + curve-angle, -0.42em)
            } else if "nat" in arr.options {
              move(dy: 0.18em, _center(
                box(clip:true, height: 0.36em, $arrow.t.double$)
              ))
              aend = v-add-dir(aend, angle + curve-angle, -0.36em)
            } else if "two" in arr.options {
              move(dy: 0.18em, _center(
                box(clip:true, height: 0.36em, $arrows.tt$)
              ))
              aend = v-add-dir(aend, angle + curve-angle, -0.36em)
            } else {
              move(dy: 0.15em, _center(
                box(clip:true, height: 0.3em, $arrow.t$)
              ))
              aend = v-add-dir(aend, angle + curve-angle, -0.3em)
            }
          )
        )

        // draw the arrow's start
        place(dx: astart.at(0), dy: astart.at(1),
          rotate(angle - curve-angle - 90deg, origin: top+left, 
            if "bij" in arr.options {
              if "nat" in arr.options {
                move(dy: 0.18em, _center(
                  box(clip:true, height: 0.36em, $arrow.t.double$)
                ))
                astart = v-add-dir(astart, angle - curve-angle, 0.36em)
		}else if "two" in arr.options {
                move(dy: 0.18em, _center(
                  box(clip:true, height: 0.36em, $arrows.tt$)
                ))
                astart = v-add-dir(astart, angle - curve-angle, 0.36em)
              } else {
                move(dy: 0.15em, _center(
                  box(clip: true, height: 0.3em, $arrow.t$)
                ))
                astart = v-add-dir(astart, angle - curve-angle, 0.3em)
              }
            } else if "inj" in arr.options {
		/*
		//Performin this control in the arr function gives better error message
		if ("nat" in arr.options) or ("two" in arr.options){
			panic("Hooked double-stemmed arrows are not supported.")
		}
		*/
              curve(stroke: (thickness: arr.stroke, cap: "round"),
                curve.move((0em, 0.15em)),
                curve.cubic(none, (0em, 0em), (0.15em, 0em)),
                curve.cubic(none, (0.3em, 0em), (0.3em, 0.15em))
              )
              astart = v-add-dir(astart, angle - curve-angle, 0.15em)
            } else if "def" in arr.options {
              place(dx: -0.2em, line(stroke: (thickness: arr.stroke, cap: "round"),
                                     length: 0.4em))
            }  
          )
        )

        // find the dash style
        let dash = arr.options
          .filter(opt => opt not in ("inj", "surj", "bij", "def", "nat", "two"))
          .at(0, default: none)

        // draw the arrow's stem
        
        // this should be, up to a good approximation (due to different
        // start and end tips), a parabola. See https://en.wikipedia.org/wiki/Bézier_curve
        let ctrl-length = - (v-length(v-sub(astart, aend)) / 3
                             / calc.cos(curve-angle))
        let ss = stroke(arr.stroke)
        let paint = ss.paint
        let thickness = ss.thickness
        let curve-stroke = (paint: paint, thickness: thickness, dash: dash, cap: "round")
        if "nat" in arr.options {
          let sep = 0.095em
          let astart_left = v-add-dir(astart, angle - curve-angle + 90deg, -sep);
          let aend_left = v-add-dir(aend, angle + curve-angle + 90deg, -sep);

          let astart_right = v-add-dir(astart, angle - curve-angle - 90deg, -sep);
          let aend_right = v-add-dir(aend, angle + curve-angle - 90deg, -sep);
          place(curve(stroke: curve-stroke,
            curve.move(astart_left),
            curve.cubic(v-add-dir(astart_left, 180deg + angle - curve-angle, ctrl-length), v-add-dir(aend_left, angle + curve-angle, ctrl-length), aend_left),
          ))
          place(curve(stroke: curve-stroke,
            curve.move(astart_right),
            curve.cubic(v-add-dir(astart_right, 180deg + angle - curve-angle, ctrl-length), v-add-dir(aend_right, angle + curve-angle, ctrl-length), aend_right),
          ))
	} else if "two" in arr.options {
          let sep = 0.206em
          
          let astart_left = v-add-dir(astart, angle - curve-angle + 90deg, -sep)
          let aend_left = v-add-dir(aend, angle + curve-angle + 90deg, -sep)

          let astart_right = v-add-dir(astart, angle - curve-angle - 90deg, -sep)
          let aend_right = v-add-dir(aend, angle + curve-angle - 90deg, -sep)
          place(curve(stroke: curve-stroke,
            curve.move(astart_left),
            curve.cubic(v-add-dir(astart_left, 180deg + angle - curve-angle, ctrl-length), v-add-dir(aend_left, angle + curve-angle, ctrl-length), aend_left),
          ))
          place(curve(stroke: curve-stroke,
            curve.move(astart_right),
            curve.cubic(v-add-dir(astart_right, 180deg + angle - curve-angle, ctrl-length), v-add-dir(aend_right, angle + curve-angle, ctrl-length), aend_right),
          ))
        } else {
          place(curve(stroke: curve-stroke,
            curve.move(astart),
            curve.cubic(v-add-dir(astart, 180deg + angle - curve-angle, ctrl-length), v-add-dir(aend, angle + curve-angle, ctrl-length), aend)
          ))
        }
      
        // draw the arrow's label
        let normal = (- aend.at(1) + astart.at(1), aend.at(0) - astart.at(0))
        let middle = v-add(v-mul(v-add(astart, aend), 0.5),
                           v-mul(normal, -calc.tan(curve-angle)/4))
      
        let m = measure(arr.label)
	let stem-thickness=0em
	if "nat" in arr.options {
		stem-thickness= 2*0.095em
	} else if "two" in arr.options {
		stem-thickness= 2*0.206em
	}
        let l = (
          (
            if arr.label-pos == right { -0.5 }
            else if arr.label-pos == left { 0.5 }
            else { 0 }
          ) * (
            0.5em +
            calc.abs(calc.sin(angle)) * (m.width + stem-thickness) +
            calc.abs(calc.cos(angle)) * (m.height + 0.3em + stem-thickness)
          ) + (
            if type(arr.label-pos) == length { arr.label-pos }
            else { 0pt }
          )
        )
        let lpos = v-add-dir(middle, angle - 90deg, l)
        if arr.label-pos == 0 {
          place(dx: lpos.at(0), dy: lpos.at(1), _center(
            rect(width: m.width + 0.1em, height: m.height + 0.3em, fill: white)
          ))
        }
        if debug {
          place(dx: lpos.at(0), dy: lpos.at(1), _center(
            rect(width: m.width + 0.5em, height: m.height + 0.8em,
                 radius: 0.25em, stroke: 0.5pt + red)
          ))
        }
        place(dx: lpos.at(0), dy: lpos.at(1), _center(arr.label))
      }
  
      for node in nodes {
        let coords = coords(node.pos)
        place(
          dx: coords.at(0),
          dy: coords.at(1),
          _center(node.label)
        )
        if debug {
          let m = measure(node.label)
          place(
            dx: coords.at(0),
            dy: coords.at(1),
            _center(rect(width: m.width + 2 * arr-clearance,
                         height: m.height + 2 * arr-clearance,
                         stroke: 0.5pt+red)),
          )
          place(
            dx: coords.at(0),
            dy: coords.at(1),
            _center(circle(
              radius: calc.max(m.width, m.height) / 2 + arr-clearance,
              stroke: 0.5pt+red
            )),
          )
        }
      }
    })
  }
}

