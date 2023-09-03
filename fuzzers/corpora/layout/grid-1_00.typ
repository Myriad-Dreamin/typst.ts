
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

#let cell(width, color) = rect(width: width, height: 2cm, fill: color)
#set page(width: 100pt, height: 140pt)
#grid(
  columns: (auto, 1fr, 3fr, 0.25cm, 3%, 2mm + 10%),
  cell(0.5cm, rgb("2a631a")),
  cell(100%,  forest),
  cell(100%,  conifer),
  cell(100%,  rgb("ff0000")),
  cell(100%,  rgb("00ff00")),
  cell(80%,   rgb("00faf0")),
  cell(1cm,   rgb("00ff00")),
  cell(0.5cm, rgb("2a631a")),
  cell(100%,  forest),
  cell(100%,  conifer),
  cell(100%,  rgb("ff0000")),
  cell(100%,  rgb("00ff00")),
)
