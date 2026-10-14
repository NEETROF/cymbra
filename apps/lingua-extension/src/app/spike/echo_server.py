#!/usr/bin/env python3
"""THROWAWAY spike (task 1.4): prints the Origin each request carries and answers CORS with that
origin echoed back, as a server that lists it in CYMBRA_ALLOWED_WEB_ORIGINS would.
usage: python3 echo_server.py [port] [host]   (host 0.0.0.0 to reach it from a phone on the LAN)"""
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


class Echo(BaseHTTPRequestHandler):
    def _cors(self):
        origin = self.headers.get("Origin")
        self.send_header("Access-Control-Allow-Origin", origin if origin else "*")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.send_header("Access-Control-Allow-Methods", "*")
        self.send_header("Vary", "Origin")

    def _log(self):
        print(f"[echo] {self.command} {self.path} Origin={self.headers.get('Origin')!r} "
              f"Referer={self.headers.get('Referer')!r} UA={self.headers.get('User-Agent')!r}", flush=True)

    def do_OPTIONS(self):
        self._log()
        print(f"[echo]   preflight ACRM={self.headers.get('Access-Control-Request-Method')!r} "
              f"ACRH={self.headers.get('Access-Control-Request-Headers')!r}", flush=True)
        self.send_response(204)
        self._cors()
        self.end_headers()

    def _reply(self):
        self._log()
        n = int(self.headers.get("Content-Length") or 0)
        if n:
            self.rfile.read(n)
        body = json.dumps({"method": self.command, "origin": self.headers.get("Origin"),
                           "referer": self.headers.get("Referer"),
                           "headers": dict(self.headers.items())}).encode()
        self.send_response(200)
        self._cors()
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    do_GET = _reply
    do_POST = _reply

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8765
    host = sys.argv[2] if len(sys.argv) > 2 else "127.0.0.1"
    print(f"[echo] listening on {host}:{port}", flush=True)
    ThreadingHTTPServer((host, port), Echo).serve_forever()
