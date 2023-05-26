
import http
import http.server
from http import HTTPStatus
import re

def server_main():
  class FrontEndHanlder(http.server.SimpleHTTPRequestHandler):

    def __init__(self, *args, **kwargs):
      kwargs['directory'] = '.'
      super().__init__(*args, **kwargs)

    def end_headers (self):
        self.send_header('Access-Control-Allow-Origin', '*')
        super().end_headers()

    def do_GET(self) -> None:
      print(self.path)

      if self.path.startswith('/fonts/'):
        path = self.path.replace('/fonts/', '/')
        path = re.sub(r'^.*(?=[A-Z]:)', '', path)
        print(path)
        with open(path, 'rb') as f:
            content = f.read()

        self.send_response(HTTPStatus.OK)
        self.send_header('Content-Type', 'application/octet-stream')
        self.send_header('Content-Length', len(content))
        self.end_headers()
        self.wfile.write(content)
        return

      if self.path.startswith('/workspace/'):
        self.path = self.path.replace('/workspace/', '/')

      super().do_GET()

  http.server.ThreadingHTTPServer(('127.0.0.1', 20812), FrontEndHanlder).serve_forever()


if __name__ == '__main__':
  server_main()
