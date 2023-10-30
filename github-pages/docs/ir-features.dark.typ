
#import "graphs.typ": ir-feature-graph

#set page(
  height: auto, width: auto, margin: 0.5em // , fill: rgb(13, 17, 23)
)
#set text(fill: white)
#show link: underline

#figure(
  ir-feature-graph(stroke-color: white, bg-color: rgb(13, 17, 23), light-theme: false),
  caption: [Figure: Features of the #emph("Vector Format"). ],
  numbering: none,
)
