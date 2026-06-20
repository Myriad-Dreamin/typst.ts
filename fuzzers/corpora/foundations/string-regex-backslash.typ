
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Using single/double backslash will change semantical meaning of a valid
// string escape sequence (ES).
#let remove(source, pattern) = source.replace(regex(pattern), "")
#test(remove("
", "\n"), "") // Literal newline (Line Feed) in source, ES in pattern (1 byte).
#test(remove("
", "\\n"), "") // The `\n` regex token that represents newline (2 bytes).
#test(remove("\n", "\n"), "") // ES in the source string.
#test(remove("\n", "\\n"), "") // ES in the source string.
#test(remove("\\\t\n1", "\\\\\\t\\n\\d"), "") // Proper string-based regex.
#test(remove("\\\t\n1", `\\\t\n\d`.text), "") // Better raw-based approach.
#test(remove(" word-wordle", "\bword\b"), " -wordle") // Invalid ES.
#test(remove(" word-wordle", "\\bword\\b"), " -wordle") // Valid regex tokens.