
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Test styling.
#show heading.where(level: 5): it => block(
  text(font: "Roboto", fill: eastern, it.body + [!])
)

= Heading
===== Heading 🌍
#heading(level: 5)[Heading]
