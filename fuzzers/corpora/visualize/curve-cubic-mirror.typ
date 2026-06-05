
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(height: 100pt)
#curve(
  fill: red,
  curve.move((0%, 0%)),
  curve.cubic((-4%, 4%), (54%, 46%), (50%, 50%)),
  curve.cubic(auto, (4%, 54%), (0%, 50%)),
  curve.cubic(auto, (54%, 4%), (50%, 0%)),
  curve.close(),
)