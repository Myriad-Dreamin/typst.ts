
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that line caps are taken into account for gradient fills.
#for cap in ("square", "butt", "round"){
  box(line(length: 10pt, stroke: (
    thickness: 20pt,
    paint: gradient.radial(blue, orange).sharp(4),
    cap: cap
  )), width: 30pt)
}