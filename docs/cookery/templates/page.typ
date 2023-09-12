// This is important for typst-book to produce a responsive layout
// and multiple targets.
#import "@preview/book:0.2.2": get-page-width, target, is-web-target, is-pdf-target

#let page-width = get-page-width()
#let is-pdf-target = is-pdf-target()
#let is-web-target = is-web-target()

// todo: move theme style parser to another lib file
#let theme-target = if target.contains("-") { target.split("-").at(1) } else { "light" }
#let theme-style = toml("theme-style.toml").at(theme-target)

#let is-dark-theme = theme-style.at("color-scheme") == "dark"
#let is-light-theme = not is-dark-theme

#let main-color = rgb(theme-style.at("main-color"))
#let dash-color = rgb(theme-style.at("dash-color"))

#let main-font = (
  "Charter",
  "Source Han Serif SC",
  "Source Han Serif TC",
  // typst-book's embedded font
  "Linux Libertine",
)

#let code-font = (
  "BlexMono Nerd Font Mono",
  // typst-book's embedded font
  "DejaVu Sans Mono",
)

// todo: move code theme parser to another lib file
#let code-theme-file = theme-style.at("code-theme")

#let code-extra-colors = if code-theme-file.len() > 0 {
  let data = xml(theme-style.at("code-theme")).first()

  let find-child(elem, tag) = {
    elem.children.find(e => "tag" in e and e.tag == tag)
  }

  let find-kv(elem, key, tag) = {
    let idx = elem.children.position(e => "tag" in e and e.tag == "key" and e.children.first() == key)
    elem.children.slice(idx).find(e => "tag" in e and e.tag == tag)
  }

  let plist-dict = find-child(data, "dict")
  let plist-array = find-child(plist-dict, "array")
  let theme-setting = find-child(plist-array, "dict")
  let theme-setting-items = find-kv(theme-setting, "settings", "dict")
  let background-setting = find-kv(theme-setting-items, "background", "string")
  let foreground-setting = find-kv(theme-setting-items, "foreground", "string")
  (
    bg: rgb(background-setting.children.first()),
    fg: rgb(foreground-setting.children.first()),
  )
} else {
  (
    bg: rgb(239, 241, 243),
    fg: none,
  )
}

// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(title: "Typst Book", authors: (), body) = {

  // set basic document metadata
  set document(author: authors, title: title) if not is-pdf-target

  // set web/pdf page properties
  set page(
    numbering: none, 
    number-align: center,
    width: page-width,
  )

  // remove margins for web target
  set page(
    margin: (
      // reserved beautiful top margin
      top: 20pt,
      // reserved for our heading style.
      // If you apply a different heading style, you may remove it.
      left: 20pt,
      // Typst is setting the page's bottom to the baseline of the last line of text. So bad :(.
      bottom: 0.5em,
      // remove rest margins.
      rest: 0pt,
    ),
    // for a website, we don't need pagination.
    height: auto,
  ) if is-web-target;

  // set text style
  set text(font: main-font, size: 16pt, fill: main-color, lang: "en")

  // render a dash to hint headings instead of bolding it.
  show heading : set text(weight: "regular") if is-web-target
  show heading : it => locate(loc => {
    if is-web-target {
      place(left, dx: -20pt, [
        #set text(fill: dash-color)
        #link(loc)[\#]
      ])
    }
    it
  })

  // link setting
  show link : set text(fill: dash-color)

  // math setting
  show math.equation: set text(weight: 400)

  // code block setting
  show raw: it => {
    set text(font: code-font)
    if "block" in it.fields() and it.block {
      rect(
        width: 100%,
        inset: (x: 4pt, y: 5pt),
        radius: 4pt,
        fill: code-extra-colors.at("bg"),
        [
          #set text(fill: code-extra-colors.at("fg")) if code-extra-colors.at("fg") != none
          #place(right, text(luma(110), it.lang))
          #it
        ],
      )
    } else {
      it
    }
  }

  // Main body.
  set par(justify: true)

  body
}

#let part-style = heading
