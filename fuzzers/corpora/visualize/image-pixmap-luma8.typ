
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#image(
  bytes(range(16).map(x => x * 16)),
  format: (
    encoding: "luma8",
    width: 4,
    height: 4,
  ),
  width: 1cm,
)