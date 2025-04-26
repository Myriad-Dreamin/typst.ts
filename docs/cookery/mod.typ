
#import "templates/page.typ": *


#let natural-image(img) = context {
  let (width, height) = measure(img)
  if width > 0pt {
    layout(page => {
      let width_scale = 0.8 * page.width / width
      block(width: width_scale * width, height: width_scale * height)[
        #scale(x: width_scale * 100%, y: width_scale * 100%, origin: center + top)[#img]
      ]
    })
  }
}

#let cond-image(img) = context if shiroa-sys-target() == "html" {
  html.elem("div", attrs: ("class": "pseudo-image"), html.frame(img))
} else {
  natural-image(img)
}
