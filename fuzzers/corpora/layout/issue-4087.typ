
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Weak space at the end of the line is removed.
This is the first line #h(2cm, weak: true) A new line

// Non-weak space consumes a specified width and pushes to next line.
This is the first line #h(2cm, weak: false) A new line

// Similarly, weak space at the beginning of the line is removed.
This is the first line \ #h(2cm, weak: true) A new line

// Non-weak-spacing, on the other hand, is not removed.
This is the first line \ #h(2cm, weak: false) A new line