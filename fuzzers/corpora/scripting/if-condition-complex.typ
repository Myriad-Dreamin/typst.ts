
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Braced condition.
#if {true} [
  One.
]

// Content block in condition.
#if [] != none [
  Two.
]

// Multi-line condition with parens.
#if (
  1 + 1
    == 1
) [
  Nope.
] else {
  "Three."
}

// Multiline.
#if false [
  Bad.
] else {
  let point = "."
  "Four" + point
}

// Content block can be argument or body depending on whitespace.
#{
  if content == type[b] [Fi] else [Nope]
  if content == type [Nope] else [ve.]
}

#let i = 3
#if i < 2 [
  Five.
] else if i < 4 [
  Six.
] else [
  Seven.
]