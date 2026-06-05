
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test that smartquotes can open before non-whitespace if not nested.
"Hello"/"World" \
'"Hello"/"World"' \
""Hello"/"World""