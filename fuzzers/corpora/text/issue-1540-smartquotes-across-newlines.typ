
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that smart quotes are inferred correctly across newlines.
"test"#linebreak()"test"

"test"\
"test"