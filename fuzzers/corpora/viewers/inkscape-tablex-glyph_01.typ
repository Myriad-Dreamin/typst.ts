#import "@preview/tablex:0.0.5": *

*Test*

test

deeteeeeereeeedetteeeee
// vvvv causes the dreaded warning (alongside another table downwards)
#tablex(
    columns: (auto, auto, auto), // rows: ((1em, 1em, 1em),) eeee
    rows: (auto,),
    column-gutter: 1fr,
    row-gutter: none,
    repeat-header: (3, 4),
    header-hlines-have-priority: false,
    align: (column, row) => {(top, center).at(calc-mod(row + column, 2))},
    // fill: (column, row) => {(blue, red).at(calc-mod(row + column, 2))},
    vlinex(), vlinex(), vlinex(), vlinex(),
    hlinex(),
    [*My*], colspanx(2)[*Headedr*],  //
    hlinex(start: 0, end: 1),
    cellx(colspan: 2, rowspan: 2)[a], [b\ c],
    hlinex(),
    () , (), [cefdsrdeefffeerddeeeeeedeeeeeeerd],
    hlinex(),
    [a], [b], [xyz],
    hlinex(end: 1),
    [b],
    hlinex(),
    ..range(0, 125).map(i => ([d], [#{i + 3}], [a],
    hlinex())).flatten(),
    [b], [c],
)

#tablex(
    columns: 5,
    rows: 1,
    stroke: red + 2pt,
    vlinex(), (), vlinex(), vlinex(), vlinex(), vlinex(),
    hlinex(),
    [abcdef], colspanx(3, rowspanx(2, [ee], fill: red), align: horizon), (), (), [c],
    hlinex(stroke: blue),
    [abcdef], (), (), (), [c],
    hlinex(),
    [aa], [b], [c], [b], cellx(inset: 2pt, align: center+horizon)[cdeecfeeeeeeeeeeeeeeeeeerdteeettetteeefdxeeeeeddeeeetec],
    hlinex(),
    // [abcdef], [a], [b],
    // hlinex(),
)

#tablex(
    columns: 4,
    [a], [b], [c], [d],
    hlinex(),
    [a], colspanx(2, rowspanx(2)[b]), [d],
    [a], (), (), [d],
    [a], [b], [c], [d],
)

#tablex(
    columns: (1fr, 1fr, 1fr, 1fr),
    map-cells: cell => (..cell, content: cell.content + [adf]),
    map-rows: (row, cells) => cells.map(c => if c == none { none } else { (..c, content: c.content + [#row]) }),
    map-cols: (col, cells) => cells.map(c => if c == none { none } else { (..c, content: c.content + [#col]) }),
    map-hlines: h => (..h, stroke: 5pt + (red, blue).at(calc-mod(h.y, 2))),
    map-vlines: v => (..v, stroke: 5pt + (yellow, green.darken(50%)).at(calc-mod(v.x, 2))),
    [a], [b], [c], [d],
    hlinex(),
    [a], colspanx(2, rowspanx(2)[b]), [d],
    [a], (), (), [d],
    [a], [b], [c], [de],
)

#tablex(
    columns: (1em, 2em, auto, auto),
    rows: (1em, 1em, auto),
    [a], [b], [cd], [d],
    hlinex(),
    [a], colspanx(2, rowspanx(2)[bcccccccc\ c\ c\ c]), [d],
    [a], (), (), [d],
    [a], (x, y) => text(size: 7pt)[#(x, y)], [f], [dee],
    [a], [b], [c], [dee],
)

eeeedreetetdeederfttddeerreddeeeteeeeeerettededteeedeceesdeedeeefteetdedeeesefdferreeedeefeettgederedaeeteeeeddrdfeeedeeffteeeeeeeeesedteteestderedeeeeefeeeeessdeeee

s
s
s
s

s

s

#tablex(
    columns: (1em, 2em, auto, auto),
    rows: (1em, 1em, auto),
    gutter: 20pt,
    align: center + horizon,
    auto-lines: true,
    map-hlines: h => (..h, stop-pre-gutter: default-if-auto(h.stop-pre-gutter, true)),
    map-vlines: h => (..h, stop-pre-gutter: default-if-auto(h.stop-pre-gutter, true)),
    [a], [b], [cd], [d],
    hlinex(start: 0, end: 1),
    hlinex(start: 4, end: 3),
    hlinex(start: 1, end: none, gutter-restrict: top),
    hlinex(start: 1, end: 2, stop-pre-gutter: false, gutter-restrict: bottom), hlinex(start: 2, end: 4, stop-pre-gutter: true, gutter-restrict: bottom),
    [a],
    vlinex(gutter-restrict: left),
    vlinex(start: 0, end: 1, gutter-restrict: right),
    vlinex(start: 3, end: 4, gutter-restrict: right),
    vlinex(start: 4, end: 5, gutter-restrict: right),
    vlinex(stop-pre-gutter: false, start: 1, end: 2, gutter-restrict: right),
    vlinex(stop-pre-gutter: true, start: 2, end: 3, gutter-restrict: right),
    colspanx(2, rowspanx(2)[bcccccccc\ c\ c\ c]), [d],
    [a], (), (), [d],
    [a], (x, y) => text(size: 7pt)[#(x, y)], [f], [dee],
    [a], [b], [c], [dee],
)

== Examples from the docs
\
#tablex(
    columns: 4,
    align: center + horizon,
    auto-vlines: false,

    // indicate the first two rows are the header
    // (in case we need to eventually
    // enable repeating the header across pages)
    header-rows: 2,

    // color the last column's cells
    // based on the written number
    map-cells: cell => {
        if cell.x == 3 and cell.y > 1 {
            cell.content = {
                let value = int(cell.content.text)
                let text-color = if value < 10 {
                    red.lighten(30%)
                } else if value < 15 {
                    yellow.darken(13%)
                } else {
                    green
                }
                set text(text-color)
                strong(cell.content)
            }
        }
        cell
    },

    /* --- header --- */
    rowspanx(2)[*Username*], colspanx(2)[*Data*], (), rowspanx(2)[*Score*],
    (),                 [*Location*], [*Height*], (),
    /* -------------- */

    [John], [Second St.], [180 cm], [5],
    [Wally], [Third Av.], [160 cm], [10],
    [Jason], [Some St.], [150 cm], [15],
    [Robert], [123 Av.], [190 cm], [20],
    [Other], [Unknown St.], [170 cm], [25],
)

#tablex(
    columns: (auto, 1em, 1fr, 1fr),  // 3 columns
    rows: auto,  // at least 1 row of auto size,
    fill: red,
    align: center + horizon,
    stroke: green,
    [a], [b], [c], [d],
    [e], [f], [g], [h],
    [i], [j], [k], [l]
)

#repeat[a]
#place(bottom+right)[b]
#tablex(
    columns: 3,
    colspanx(2)[a], (),  [b],
    [c], rowspanx(2)[d], [ed],
    [f], (),             [g]
)

#tablex(
    columns: 4,
    auto-lines: false,
    vlinex(), vlinex(), vlinex(), (), vlinex(),
    colspanx(2)[a], (),  [b], [J],
    [c], rowspanx(2)[d], [e], [K],
    [f], (),             [g], [L],
)

#tablex(
    columns: 4,
    auto-vlines: false,
    colspanx(2)[a], (),  [b], [J],
    [c], rowspanx(2)[d], [e], [K],
    [f], (),             [g], [L],
)

#block(breakable: false, gridx(
    columns: 4,
    (), (), vlinex(end: 2),
    hlinex(stroke: yellow + 2pt),
    colspanx(2)[a], (),  [b], [J],
    hlinex(start: 0, end: 1, stroke: yellow + 2pt),
    hlinex(start: 1, end: 2, stroke: green + 2pt),
    hlinex(start: 2, end: 3, stroke: red + 2pt),
    hlinex(start: 3, end: 4, stroke: blue.lighten(50%) + 2pt),
    [c], rowspanx(2)[d], [e], [K],
    hlinex(start: 2),
    [f], (),             [g], [L],
))

#block(breakable: false, tablex(
    columns: 3,
    map-hlines: h => (..h, stroke: blue),
    map-vlines: v => (..v, stroke: green + 2pt),
    colspanx(2)[a], (),  [b],
    [c], rowspanx(2)[d], [ed],
    [f], (),             [g]
))

#block(breakable: false, tablex(
    columns: 3,
    fill: red,
    align: right,
    colspanx(2)[a], (),  [beeee],
    [c], rowspanx(2)[d], cellx(fill: blue, align: left)[e],
    [f], (),             [g],

    // place this cell at the first column, seventh row
    cellx(colspan: 3, align: center, x: 0, y: 6)[hi I'm down here]
))

#tablex(
    columns: 4,
    auto-vlines: true,

    // make all cells italicized
    map-cells: cell => {
        (..cell, content: emph(cell.content))
    },

    // add some arbitrary content to entire rows
    map-rows: (row, cells) => cells.map(c =>
        if c == none {
            c
        } else {
            (..c, content: [#c.content\ *R#row*])
        }
    ),

    // color cells based on their columns
    // (using 'fill: (column, row) => color' also works
    // for this particular purpose)
    map-cols: (col, cells) => cells.map(c =>
        if c == none {
            c
        } else {
            (..c, fill: if col < 2 { blue } else { yellow })
        }
    ),

    colspanx(2)[a], (),  [b], [J],
    [c], rowspanx(2)[dd], [e], [K],
    [f], (),             [g], [L],
)

#tablex(
    columns: 4,
    fill: blue,
    colspanx(2, rotate(30deg)[a]), rotate(30deg)[a], rotate(30deg)[a],rotate(30deg)[a],
)

#tablex(
    columns: 4,
    stroke: 5pt,
    fill: blue,
    (), vlinex(expand: (-2%, 4pt)),
    [a], [b], [c], [d],
    [e], [f], [g], [h]
)

#set page(width: 300pt)
#pagebreak()
#v(80%)

#tablex(
    columns: 4,
    align: center + horizon,
    auto-vlines: false,
    repeat-header: true,
    header-rows: 2,

    /* --- header --- */
    rowspanx(2)[*Names*], colspanx(2)[*Properties*], (), rowspanx(2)[*Creators*],
    (),                 [*Type*], [*Size*], (),
    /* -------------- */

    [Machine], [Steel], [5 $"cm"^3$], [John p& Kate],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Robert],
    [Frog], [Animal], [6 $"cm"^3$], [Rodbert],
)

#v(35em)
#set page(width: auto, height: auto)

*Auto page tests (infinite dimensions):*

#table(
    columns: 3,
    [a], [b], [c],
    [d], [e], [f],
    [g], [h], [i],
    [f], [j], [e\ b\ c\ d],
)

#tablex(
    columns: 3,
    [a], [b], [c],
    [d], [e], [f],
    [g], [h], [i],
    [f], [j], [e\ b\ c\ d],
)

#table(
    columns: (99%, auto),
    [a], [b],
    [c], [d]
)

#tablex(
    columns: (99%, auto),
    [a], [b],
    [c], [d]
)

#table(
    columns: (auto, 1fr, 1fr),
    [a], [b], [c],
    [c], [d], [e]
)

#tablex(
    columns: (auto, 1fr, 1fr),
    [a], [b], [c],
    [c], [d], [e]
)

#table(
    columns: 4,
    gutter: 10pt,
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
)

// vvv causes the dreaded warning (alongside the first table in the file)
#tablex(
    columns: 4,
    gutter: 10pt,
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
)

#set page(width: 300pt, height: 1000pt)

#tablex(
    columns: (1fr, 1fr, 1fr),
    [a], [b], [c]
)

#table(
    columns: (1fr, 1fr, 1fr),
    [a], [b], [c]
)

#tablex(
    columns: (10%, 10%, 10%, 10%, 10%),
    // map-hlines: h => (..h, stop-pre-gutter: default-if-auto(h.stop-pre-gutter, true)),
    // map-vlines: v => (..v, stop-pre-gutter: default-if-auto(v.stop-pre-gutter, true)),
    gutter: 15pt,
    [a], [b], [c], [d], [e],
    hlinex(stroke: blue),
    [f], rowspanx(2, colspanx(2)[ggggoprdeetet\ eeeeeee]), (), [i], [j],
    [k], (), (), [n], [o],
    [p], [q], [r], [s], [t]
)

#tablex(
    columns: (auto, 1fr),
    rowspanx(2, [a\ a\ a\ a\ a]), "dfjasdfjdaskfjdsaklfj", "height should be correct here"
)

This table should be contained within the page's width:
#tablex(
    columns: (auto, auto),
    [#lorem(40)], [#lorem(100)]
)

Accept array of column alignments:
#block(breakable: false, tablex(
    columns: 5,
    align: (right + top, center + bottom, left + horizon),
    [a], [b], [d], [e], [f],
    [cccc], [cccfdd], [esdfsd], [ffeff\ erfad], [adspfp]
))
Empty array inherits from outside:
#block(breakable: false, tablex(
    columns: 5,
    align: (),
    [a], [b], [d], [e], [f],
    [cccc], [cccfdd], [esdfsd], [ffeff\ erfad], [adspfp]
))

Accept array for fill:
#tablex(
    columns: 5,
    fill: (red, blue, green),
    [a], [b], [c], [d], [e],
    [dddd], [eeee], [ffff], [ggggg], [hhhhhh]
)

Empty fill array is no-op:
#tablex(
    columns: 5,
    fill: (),
    [a], [b], [c], [d], [e],
    [dddd], [eeee], [ffff], [ggggg], [hhhhhh]
)

Align and fill function tests:
#tablex(
    columns: 5,
    align: (column, row) => (
        (top, bottom).at(row)
        + (left, right).at(calc-mod(column, 2))
    ),
    fill: (column, row) => (red, blue).at(row).lighten((50%, 10%).at(calc-mod(column, 2))),
    [a\ b], [b], [c], [d], [e],
    [dddd\ eapdsfp], [eeee\ eapdlf], [ffff], [ggggg], [hhhhhh]
)

Test division by zero bug:
#tablex(
	columns: 3,

	[Name],[Entität],[Eigenschaft],
	[GammaTaurus],[ThisIsASuperlongSymbolicName which is similar important as Supercalifragilistic],[],
)

Test superfluous row bug:
#tablex(
  columns: 3,
  [a],
  cellx(y: 2)[a]
)

Test gutter restrict top:
#tablex(
    columns: 3,
    auto-lines: false,
    row-gutter: 5pt,
    [a], [b], [c],
    hlinex(gutter-restrict: top),
    hlinex(gutter-restrict: bottom),
    [d], [e], [f]
)

Test gutter restrict without gutter:
#tablex(
    columns: 3,
    auto-lines: false,
    [a], [b], [c],
    hlinex(gutter-restrict: top),
    [e], [f], [g],
    hlinex(gutter-restrict: bottom),
    [d], vlinex(gutter-restrict: left), [e], vlinex(gutter-restrict: right), [f]
)

#pagebreak(weak: true)

#v(80%)

Test gutter split between pages:

#tablex(
    columns: 3,
    auto-vlines: false,
    row-gutter: 5pt,
    [a], [b], [c],
    [a], [b], [c],
    [a], [b], [c],
    [a], [b], [c],
    [a], [b], [c],
    [a], [b], [c],
    [a], [b], [c],
    hlinex(stroke: blue),
    [a], [b], [c],
    [a], [b], [c],
)

Small gutter test:

#tablex(
    columns: 4,
    gutter: 10pt,
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
    [a], [b], [c], [d],
)

Test fractional columns in an auto-sized block:

#block(tablex(
    columns: (auto, 1fr, 1fr),
    [a], [b], [c],
    [d], [e], [f],
    [g], [h], [i]
))

*Using the examples from issue \#44:*

1.
#table(columns: 1fr, [1A. table])
#tablex(columns: 1fr, [1B. tablex])

2.
#block(table(columns: 1fr, [2A. table plain block]))
#block(tablex(columns: 1fr, [2B. tablex plain block]))

3.
#block(breakable: true, table(columns: 1fr, [3A. table breakable: true]))
#block(breakable: true, tablex(columns: 1fr, [3B. tablex breakable: true]))

4.
#block(breakable: false, table(columns: 1fr, [4A. table breakable: false]))
#block(breakable: false, tablex(columns: 1fr, [4B. tablex breakable: false]))

*Nested tables from issue \#41:*

- Triple-nested tables.

#tablex(
  tablex(
    tablex(
      lorem(10)
    )
  )
)

- Quadruple-nested tables.

#tablex(
  tablex(
    tablex(
      tablex(
        lorem(20)
      )
    )
  )
)

*Nested tables from issue \#28:*

#let mycell = [
  #tablex(
    columns: (1fr, 1fr),
    [A],[A]
  )
]

= table inside a table
#tablex(
  columns: (1fr, 1fr),
  mycell, mycell
)

= following table fails
*Problem/Observation*:  just one column "C"

*Expected Outcome*: Two columns

#tablex(
  columns: (1fr, 1fr),
  [C],[C]
)

*Exotic strokes from issue \#49:*

#tablex(
    stroke: 1em,
    [C], [C]
)

// Uncomment after minimum typst version is raised enough for this
// #let s = rect(stroke: (thickness: 1em, miter-limit: 5.0)).stroke
// #tablex(
//     stroke: s,
//     [C], [C]
// )
