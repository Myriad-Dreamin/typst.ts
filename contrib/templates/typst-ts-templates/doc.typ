
#import "lib.typ": doc-project

#import "@preview/doc:0.1.0": parse-module2, show-module

// Take a look at the file `template.typ` in the file panel
// to customize this template and discover how it works.
#show: doc-project.with(
  title: "templates",
  subtitle: "document/page for typst.ts.",
  authors: (
    "Myriad-Dreamin",
  ),
  // Insert your abstract after the colon, wrapped in brackets.
  // Example: `abstract: [This is my abstract...]`
  abstract: [document/page for typst.ts. ],
  date: "2023",
)

// We can apply global styling here to affect the looks
// of the documentation. 
#set text(font: "DM Sans")
#show heading.where(level: 1): it => {
  align(center, it)
}
#show heading: set text(size: 1.5em)
#show heading.where(level: 3): set text(size: .7em, style: "italic")


#{
  let template-module = parse-module2("lib.typ", read("lib.typ"), name: "Typst.ts Templates")

  show-module(template-module, first-heading-level: 1)
  
  // Also show the "complex" sub-module which belongs to the main module (funny-math.typ) since it is imported by it. 
  // show-module(template-module-ext, show-module-name: false, first-heading-level: 1)
}
