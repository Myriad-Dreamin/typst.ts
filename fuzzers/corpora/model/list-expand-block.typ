
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Lists should shrink to fit their own items inside `auto`-width blocks,
// or expand to the full width of fixed-width containers (/page).
#block[
  - #align(center)[a]
  - bbbb
  - #rect(width: 4em, height: 1em, fill: red)
]

#block(width: 6em)[
  - #align(center)[a]
  - bbbb
  - #rect(width: 4em, height: 1em, fill: red)
]

- #align(center)[a]
- bbbb
- #rect(width: 4em, height: 1em, fill: red)