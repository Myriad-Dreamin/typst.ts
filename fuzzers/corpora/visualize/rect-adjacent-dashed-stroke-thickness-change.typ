
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
#rect(
  height: 1.2cm,
  width: 1.5cm,
  stroke: (
    bottom: (thickness: 4pt, dash: "loosely-dashed"),
    left: (thickness: 8pt, dash: "loosely-dashed"),
  ),
)