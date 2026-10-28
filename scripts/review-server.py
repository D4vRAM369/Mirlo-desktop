# Servidor estático con CORS, solo para revisar el diseño en el navegador integrado.
import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

class H(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', 'http://localhost:1420')
        super().end_headers()

ThreadingHTTPServer(('127.0.0.1', int(sys.argv[1])), partial(H, directory=sys.argv[2])).serve_forever()
