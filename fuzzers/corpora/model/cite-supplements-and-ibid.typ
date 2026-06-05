
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: 300pt)

Par 1 @arrgh

Par 2 @arrgh[p. 5-8]

Par 3 @arrgh[p. 5-8]

Par 4 @arrgh[p. 9-10]

Par 5 @arrgh[*p. 9-10*]

Par 6 @arrgh[*p. 9-10*]

#let style = bytes(
  ```xml
  <?xml version="1.0" encoding="utf-8"?>
  <style xmlns="http://purl.org/net/xbiblio/csl" version="1.0" class="note" default-locale="pl-PL">
    <info>
      <title>Example citation style</title>
      <id>http://www.example.com/</id>
    </info>
    <macro name="locator">
      <group delimiter=" ">
        <label variable="locator" form="short"/>
        <text variable="locator"/>
      </group>
    </macro>

    <citation>
      <sort>
        <key variable="title"/>
      </sort>
      <layout>
        <choose>
          <if position="first">
            <group delimiter=", ">
              <text variable="title"/>
              <text macro="locator"/>
            </group>
          </if>
          <else-if position="ibid-with-locator">
            <group delimiter=", ">
              <text term="ibid"/>
              <text macro="locator"/>
            </group>
          </else-if>
          <else-if position="ibid">
            <text term="ibid"/>
          </else-if>
          <else-if position="subsequent">
            <group delimiter=", ">
              <text variable="title"/>
              <text macro="locator"/>
            </group>
          </else-if>
        </choose>
      </layout>
    </citation>

    <bibliography>
      <sort>
        <key variable="title"/>
      </sort>
      <layout>
        <text variable="title"/>
      </layout>
    </bibliography>
  </style>
  ```.text
)

#bibliography("/assets/bib/works.bib", style: style)