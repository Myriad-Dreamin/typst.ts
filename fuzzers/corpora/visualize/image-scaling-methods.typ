
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let img(scaling) = image(
  bytes((
    0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF,
    0x80, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80,
    0x80, 0x80, 0x00, 0x00, 0x80, 0x80, 0x80, 0x00, 0x80,
  )),
  format: (
    encoding: "rgb8",
    width: 3,
    height: 3,
  ),
  width: 1cm,
  scaling: scaling,
)

#let images = (
  img(auto),
  img("smooth"),
  img("pixelated"),
)

#context if target() == "html" {
  // TODO: Remove this once `stack` is supported in HTML export.
  html.div(
    style: "display: flex; flex-direction: row; gap: 4pt",
    images.join(),
  )
} else {
  stack(
    dir: ltr,
    spacing: 4pt,
    ..images,
  )
}