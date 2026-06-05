
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 180pt)
#set text(6pt)
#set raw(lang:"python")

Inline raws, multiline e.g. `for i in range(10):
  # Only this line is a comment.
  print(i)` or otherwise e.g. `print(j)`, are colored properly.

Inline raws, multiline e.g. `
# Appears blocky due to linebreaks at the boundary.
for i in range(10):
  print(i)
` or otherwise e.g. `print(j)`, are colored properly.