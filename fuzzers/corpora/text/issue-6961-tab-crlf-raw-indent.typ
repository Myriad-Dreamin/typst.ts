
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#let snippet = (
  ```
  A
    BC
    D
  ```
)

#raw(
  snippet.text.replace("  ", "\t").replace("\n", "\r\n"),
  block: true,
)