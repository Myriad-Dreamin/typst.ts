
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Lists should shrink to fit their own contents inside auto-width pages.
#set page(width: auto)
- #align(center)[a]
- #rect(width: 4em, height: 1em, fill: red)

longlonglonglonglonglonglong