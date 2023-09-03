
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test colored delimiters
$ lr(
    text("(", fill: #green) a/b
    text(")", fill: #blue)
  ) $
