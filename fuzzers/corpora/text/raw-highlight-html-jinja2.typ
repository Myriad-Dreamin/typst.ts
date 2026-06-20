
#import "/contrib/templates/std-tests/preset.typ": *
#show: test-page
#set page(width: auto)
// FUTURE: Convert to raw syntax when we re-enable non-identifier language tags

#raw(block: true, lang: "html.j2", ```
<tbody>
  {% for row in data.rows %}
  <tr>
      {% for column in row %}
      <td>{{ column }}</td>
      {% endfor %}
  </tr>
  {% endfor %}
</tbody>
```.text)