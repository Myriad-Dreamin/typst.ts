
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page

// Layout inside a block with certain dimensions should provide those dimensions.

#set page(height: 120pt)
#block(width: 60pt, height: 80pt, layout(size => [
  This block has a width of #size.width and height of #size.height
]))
