
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Heading vs. no heading.

// Parsed as headings if at start of the context.
/**/ = Level 1
#[== Level 2]
#box[=== Level 3]

// Not at the start of the context.
No = heading

// Escaped.
\= No heading
