
#let node(pos, label) = (kind: "node", pos: pos, label: label)

#let arr(start, end, label, start_space: none, end_space: none, label_pos: 1em, curve: 0deg, stroke: 0.45pt, ..options) = {
  (
    kind: "arrow",
    start: start,
    end: end,
    label: label,
    start_space: start_space,
    end_space: end_space,
    label_pos: label_pos,
    curve: curve,
    stroke: stroke,
    options: options.pos(),
  )
}

#let commutative_diagram(
  node_padding: (70pt, 70pt),
  arr_clearance: 0.7em,
  padding: 1.5em,
  debug: false,
  bg-color: rgb("#fff"),
  ..entities
) = {
  style(styles => {

    // useful utility function
    let center(c) = pad(left: -100%, top: -100%, c)
  
    // get nodes and arrows from the function arguments
    let entities = entities.pos()
    let nodes = entities.filter(e => e.kind == "node")
    let arrows = entities.filter(e => e.kind == "arrow")
  
    // adjust node and arrow coordinates so that the min is (0, 0)
    let min_row = calc.min(..nodes.map(node => node.pos.at(0)))
    let min_col = calc.min(..nodes.map(node => node.pos.at(1)))
    let max_row = calc.max(..nodes.map(node => node.pos.at(0)))
    let max_col = calc.max(..nodes.map(node => node.pos.at(1)))
  
    for node in nodes {
      node.pos.at(0) -= min_row
      node.pos.at(1) -= min_col
    }
  
    for arr in arrows {
      arr.start.at(0) -= min_row
      arr.start.at(1) -= min_col
      arr.end.at(0) -= min_row
      arr.end.at(1) -= min_col
    }
  
    max_row -= min_row
    max_col -= min_col
    min_col = 0
    min_row = 0
  
    // measure the size of the various rows and columns
    let col_sizes = ()
    let row_sizes = ()
    for r in range(0, max_row+1) {
      row_sizes.push(0pt)
    }
    for c in range(0, max_col+1) {
      col_sizes.push(0pt)
    }
    for node in nodes {
      let m = measure(node.label, styles)
      if m.width > col_sizes.at(node.pos.at(1)) {
        col_sizes.at(node.pos.at(1)) = m.width
      }
      if m.height > row_sizes.at(node.pos.at(0)) {
        row_sizes.at(node.pos.at(0)) = m.height
      }
    }
    let row_pos = row_sizes
    let col_pos = col_sizes
    row_pos.at(0) /= 2
    col_pos.at(0) /= 2
    for r in range(1, max_row+1) {
      row_pos.at(r) += row_pos.at(r - 1) + node_padding.at(1)
    }
    for c in range(1, max_col+1) {
      col_pos.at(c) += col_pos.at(c - 1) + node_padding.at(0)
    }
  
    // total diagram dimensions
    let height = row_sizes.fold(-node_padding.at(1), (x, y) => x+y+node_padding.at(1))
    let width = col_sizes.fold(-node_padding.at(0), (x, y) => x+y+node_padding.at(0))

    // useful functions used many times later
    let coords(pos) = (
      col_pos.at(pos.at(1)), row_pos.at(pos.at(0))
    )
  
    let size_at(pos) = {
      for node in nodes {
        if node.pos == pos {
          return measure(node.label, styles)
        }
      }
      return (width: 0pt, height: 0pt)
    }
  
    // various vector utility functions
    let v_add(a, b) = (a.at(0) + b.at(0), a.at(1) + b.at(1))
    let v_sub(a, b) = (a.at(0) - b.at(0), a.at(1) - b.at(1))
    let v_mul(a, x) = (a.at(0) * x, a.at(1) * x)
    let v_add_dir(p, a, l) = v_add(p, v_mul((calc.cos(a), calc.sin(a)), l))
    let v_length(vv) = calc.sqrt(vv.at(0)/1mm*vv.at(0)/1mm + vv.at(1)/1mm*vv.at(1)/1mm)*1mm
  
    // measures the length of a segment starting in the center
    // of a rectangle and ending on one of the rectangle's edges,
    // given the angle of the segment
    let measure_at_angle(m, a, padding) = {
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
  
    let pt_per_em = measure(rect(width: 1em), styles).width

    // the box where all the diagram's elements are
    box(
      width: width + 2*padding, height: height + 2*padding,
      stroke: if debug { 0.5pt + red } else { none },
      inset: padding,
      {
      for arr in arrows {
        let start = coords(arr.start)
        let end = coords(arr.end)
        let angle = calc.atan2((end.at(0) - start.at(0)) / 1pt, (end.at(1) - start.at(1)) / 1pt)
        let curve_angle = arr.curve
        let start_space = arr.start_space
        let end_space = arr.end_space
        if start_space == none {
          start_space = measure_at_angle(size_at(arr.start), angle - curve_angle, arr_clearance / 1em * pt_per_em) / pt_per_em * 1em
        }
        if end_space == none {
          end_space = measure_at_angle(size_at(arr.end), angle + curve_angle, arr_clearance / 1em * pt_per_em) / pt_per_em * 1em
        }
        if "inj" in arr.options {
          start_space += 0.2em
        }
        let astart = v_add_dir(start, angle - curve_angle, start_space)
        let aend = v_add_dir(end, angle + 180deg + curve_angle, end_space)
      
        // more hacky, but more beautiful
        place(dx: aend.at(0), dy: aend.at(1),
          rotate(angle + curve_angle + 90deg, origin: top+left,
            if "surj" in arr.options {
              move(dy: 0.3em, center(box[
                $arrow.t.twohead$
                #place(dy: -0.35em, dx: 0.24em, rect(width: 0.10em, height: 0.7em, fill: bg-color))
              ]))
            } else {
              move(dy: 0.3em, center(box[
                $arrow.t$
                #place(dy: -0.50em, dx: 0.2em, rect(width: 0.10em, height: 0.7em, fill: bg-color))
              ]))
            }
          )
        )

        place(dx: astart.at(0), dy: astart.at(1),
          rotate(angle - curve_angle - 90deg, origin: top+left, {
            if "bij" in arr.options {
              move(dy: 0.25em, center(box[
              $arrow.t$
              #place(dy: -0.50em, dx: 0.20em, rect(width: 0.10em, height: 0.7em, fill: bg-color))
            ]))
            } else if "inj" in arr.options {
              place(pad(top: -100%, circle(stroke: arr.stroke, radius: 0.15em))) + rect(width: 0.33em, height: 0.2em, fill: bg-color)
            } else if "def" in arr.options {
              place(dx: -0.2em, line(stroke: arr.stroke, length: 0.4em))
            }  
            })
        )
      
        let N = int(20*calc.abs(curve_angle / 1rad) + 1)
        let frac = 1
        if "dashed" in arr.options {
          N = int((v_length(v_sub(start, end)) - (start_space + end_space) / 1em * pt_per_em) / 8pt)
          frac = 0.7
        }
        let normal = (- aend.at(1) + astart.at(1), aend.at(0) - astart.at(0))
        let t = calc.tan(curve_angle)
        for i in range(-N, N) {
          place(line(
            start: v_add(v_add(v_mul(astart, (N -i)/(2*N)), v_mul(aend, (N+i)/(2*N))),
              v_mul(normal, (i - N)*(i + N)/N/N/4*t)),
            end: v_add(v_add(v_mul(astart, (N -i -frac)/(2*N)), v_mul(aend, (N+i+frac)/(2*N))),
              v_mul(normal, (i +frac - N)*(i+frac + N)/N/N/4*t)),
            stroke: arr.stroke,
          ))
        }
      
        let middle = v_add(v_mul(v_add(astart, aend), 0.5), v_mul(normal, -t/4))
      
        if arr.label_pos == 0 {
          place(dx: middle.at(0), dy: middle.at(1), center(rect(fill: bg-color, arr.label, outset:-0.2em)))
        } else {
          let lpos = v_add_dir(middle, angle - 90deg, arr.label_pos)
          place(dx: lpos.at(0), dy: lpos.at(1), center(arr.label))
        }
      }
  
      for node in nodes {
        let coords = coords(node.pos)
        place(
          dx: coords.at(0),
          dy: coords.at(1),
          center(node.label)
        )
        if debug {
          let m = measure(node.label, styles)
          place(
            dx: coords.at(0),
            dy: coords.at(1),
            center(rect(width: m.width + 2 * arr_clearance, height: m.height + 2 * arr_clearance, stroke: 0.5pt+red)),
          )
          place(
            dx: coords.at(0),
            dy: coords.at(1),
            center(circle(radius: calc.max(m.width, m.height) / 2 + arr_clearance, stroke: 0.5pt+red)),
          )
        }
      }
    })
  })
}