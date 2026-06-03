
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#image(
  bytes(range(16).map(x => (0x80, x * 16)).flatten()),
  format: (
    encoding: "lumaa8",
    width: 4,
    height: 4,
  ),
  width: 1cm,
)