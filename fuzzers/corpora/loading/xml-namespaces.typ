
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
// Test reading XML data containing namespaces.
#test(
  xml(bytes(
    ```xml
    <data xmlns="http://example.org" xmlns:foo="urn:foo">
      <hello name="hi">1</hello>
      <foo:hello>World</foo:hello>
    </data>
    ```.text
  )),
  ((
    namespace: "http://example.org",
    tag: "data",
    attrs: (:),
    children: (
      "\n  ",
      (namespace: "http://example.org", tag: "hello", attrs: (name: "hi"), children: ("1",)),
      "\n  ",
      (namespace: "urn:foo", tag: "hello", attrs: (:), children: ("World",)),
      "\n",
    ),
  ),),
)