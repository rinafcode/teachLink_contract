import json
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path


DATA_FILE = Path("/data/last-alert.json")


class AlertHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status":"ok"}')
            return

        if self.path == "/alerts":
            payload = {"alerts": []}
            if DATA_FILE.exists():
                payload = json.loads(DATA_FILE.read_text(encoding="utf-8"))

            body = json.dumps(payload).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        if self.path != "/alerts":
            self.send_response(404)
            self.end_headers()
            return

        length = int(self.headers.get("Content-Length", "0"))
        raw_body = self.rfile.read(length)
        payload = json.loads(raw_body or b"{}")
        payload["receivedAt"] = datetime.now(timezone.utc).isoformat()
        DATA_FILE.write_text(json.dumps(payload, indent=2), encoding="utf-8")

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b'{"status":"received"}')

    def log_message(self, fmt, *args):
        print("%s - - [%s] %s" % (self.client_address[0], self.log_date_time_string(), fmt % args))


if __name__ == "__main__":
    server = HTTPServer(("0.0.0.0", 5001), AlertHandler)
    print("alert-webhook listening on :5001")
    server.serve_forever()
