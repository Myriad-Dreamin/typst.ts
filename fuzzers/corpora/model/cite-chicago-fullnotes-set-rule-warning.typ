
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test warning for deprecated alias.
// Warning: 18-37 style `"chicago-fullnotes"` has been deprecated in favor of `"chicago-notes"`
#set cite(style: "chicago-fullnotes")