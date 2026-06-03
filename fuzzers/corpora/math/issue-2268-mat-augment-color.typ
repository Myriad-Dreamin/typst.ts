
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// The augment line should be of the same color as the text
#set text(
  font: "New Computer Modern",
  lang: "en",
  fill: yellow,
)

$mat(augment: #1, M, v) arrow.r.squiggly mat(augment: #1, R, b)$