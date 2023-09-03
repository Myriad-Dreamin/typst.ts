
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Trimming.

// Space between "rust" and "let" is trimmed.
The keyword ```rust let```.

// Trimming depends on number backticks.
(``) \
(` untrimmed `) \
(``` trimmed` ```) \
(``` trimmed ```) \
(``` trimmed```) \
