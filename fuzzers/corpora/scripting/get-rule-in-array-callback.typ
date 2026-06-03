
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test whether context is retained in built-in callback.
#set text(lang: "de")
#context test(
  ("en", "de", "fr").sorted(key: v => v != text.lang),
  ("de", "en", "fr"),
)