
#import "data-flow.typ": data-flow-graph

#set page(height: auto, width: auto, margin: 0.5em)
#show link: underline

#figure(
  data-flow-graph(stroke: black, bg-color: white, light-theme: true),
  caption: [Browser-side module needed: $dagger$: compiler; $dagger.double$: renderer. ],
  numbering: none,
)
