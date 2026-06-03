
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test complex stroke fields.
#let r1 = rect(stroke: (paint: cmyk(1%, 2%, 3%, 4%), thickness: 4em + 2pt, cap: "round", join: "bevel", miter-limit: 5.0, dash: none))
#let r2 = rect(stroke: (paint: cmyk(1%, 2%, 3%, 4%), thickness: 4em + 2pt, cap: "round", join: "bevel", miter-limit: 5.0, dash: (3pt, "dot", 4em)))
#let r3 = rect(stroke: (paint: cmyk(1%, 2%, 3%, 4%), thickness: 4em + 2pt, cap: "round", join: "bevel", dash: (array: (3pt, "dot", 4em), phase: 5em)))
#let s1 = r1.stroke
#let s2 = r2.stroke
#let s3 = r3.stroke
#test(s1.paint, cmyk(1%, 2%, 3%, 4%))
#test(s1.thickness, 4em + 2pt)
#test(s1.cap, "round")
#test(s1.join, "bevel")
#test(s1.miter-limit, 5.0)
#test(s3.miter-limit, auto)
#test(s1.dash, none)
#test(s2.dash, (array: (3pt, "dot", 4em), phase: 0pt))
#test(s3.dash, (array: (3pt, "dot", 4em), phase: 5em))