
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// CMYK is excluded here and tested separately below because it's currently not
// 100% reproducible for SVG with the colors used in this test. (The color value
// is off by one on macOS. This might be due to different SIMD instruction sets
// being used by CMYK conversion in moxcms. We should investigate whether we can
// somehow fix this in the future.)
#set page(height: auto, margin: 0pt)
#set block(spacing: 0pt)
#let spaces = (
  ("HSV", color.hsv),
  ("HSL", color.hsl),
  ("Oklch", color.oklch),
  ("Oklab", color.oklab),
  ("sRGB", color.rgb),
  ("linear-RGB", color.linear-rgb),
  ("Luma", color.luma),
)
#for (name, space) in spaces {
  block(
    width: 100%,
    inset: 4pt,
    fill: gradient.linear(yellow, blue, space: space),
    name,
  )
}