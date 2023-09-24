
#import "data-flow.typ": data-flow-graph

#set page(height: auto, width: auto, margin: 0.5em)
#set text(fill: white)
#show link: underline

#figure(
  data-flow-graph(stroke: white, bg-color: rgb(13, 17, 23), light-theme: false),
  caption: [Browser-side module needed: $dagger$: compiler; $dagger.double$: renderer. ],
  numbering: none,
)
