
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test a combination of CSL `display` attributes. Most of the display
// attributes are barely used by any styles, so we have a custom style here.

#let style = ```csl
  <?xml version="1.0" encoding="utf-8"?>
  <style xmlns="http://purl.org/net/xbiblio/csl" class="in-text" version="1.0">
    <info>
      <title>Test</title>
      <id>test</id>
    </info>
    <citation collapse="citation-number">
      <layout>
        <text variable="citation-number"/>
      </layout>
    </citation>
    <bibliography>
      <layout>
        <text variable="title" font-style="italic" />
        <text variable="citation-number" display="left-margin" prefix="|" suffix="|" />
        <group display="indent">
          <text term="by" suffix=" " />
          <!-- This left-margin attribute is ignored because it is in a container. -->
          <names variable="author" display="left-margin" />
        </group>
        <group display="block" prefix="(" suffix=")">
          <text term="edition" suffix=" " text-case="capitalize-first" />
          <date variable="issued"><date-part name="year"/></date>
        </group>
      </layout>
    </bibliography>
  </style>
```

#let bib = ```bib
  @article{entry1,
    title={Title 1},
    author={Author 1},
    year={2021},
  }
```

#bibliography(
  bytes(bib.text),
  style: bytes(style.text),
  title: none,
  full: true,
)