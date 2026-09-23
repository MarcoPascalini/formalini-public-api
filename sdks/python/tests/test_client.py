import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError

from formalini import ApiError, Client, MalformedResponse


class Response:
    def __init__(self, payload):
        self.payload = json.dumps(payload).encode()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False

    def read(self):
        return self.payload


class ClientTests(unittest.TestCase):
    def test_list_uses_documented_host_scope_and_response(self):
        with patch("formalini.client.urlopen", return_value=Response({"documents": [{"id": "example"}]})) as send:
            documents = Client("test-key", timeout=5).documents.list(limit=10, status="approved")
        request = send.call_args.args[0]
        self.assertEqual(request.full_url, "https://api.formalini.com/api/public/v1/documents?limit=10&status=approved")
        self.assertEqual(request.get_header("Authorization"), "Bearer test-key")
        self.assertEqual(send.call_args.kwargs["timeout"], 5)
        self.assertEqual(documents[0]["id"], "example")

    def test_json_methods_construct_documented_requests(self):
        client = Client("test-key")
        cases = [
            (lambda: client.documents.create(name="Invoice", fields=[]), "POST", "/documents", {"name": "Invoice", "fields": []}),
            (lambda: client.documents.get("doc-id"), "GET", "/documents/doc-id", None),
            (lambda: client.documents.update("doc-id", status="approved", fields=[{"name": "total", "value": "4"}]), "PATCH", "/documents/doc-id", {"status": "approved", "fields": [{"name": "total", "value": "4"}]}),
            (lambda: client.workflows.dispatch(event="document.approved", document_id="doc-id"), "POST", "/workflows/dispatch", {"event": "document.approved", "document_id": "doc-id"}),
        ]
        for call, method, path, body in cases:
            with self.subTest(path=path, method=method), patch("formalini.client.urlopen", return_value=Response({"ok": True})) as send:
                call()
                request = send.call_args.args[0]
                self.assertEqual(request.get_method(), method)
                self.assertTrue(request.full_url.endswith(path))
                if body is not None:
                    self.assertEqual(json.loads(request.data), body)

    def test_upload_repeats_file_field_without_pdf(self):
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "first.jpg"
            second = Path(directory) / "second.png"
            first.write_bytes(b"jpeg-example")
            second.write_bytes(b"png-example")
            with patch("formalini.client.urlopen", return_value=Response({"document": {"id": "example"}})) as send:
                Client("test-key").documents.upload([first, second], name="Example", asynchronous=True)
            request = send.call_args.args[0]
            self.assertEqual(request.data.count(b'name="file"'), 2)
            self.assertIn(b'name="async"\r\n\r\ntrue', request.data)
            self.assertIn(b'name="name"\r\n\r\nExample', request.data)
            self.assertIn("multipart/form-data", request.get_header("Content-type"))
            with self.assertRaises(ValueError):
                Client("test-key").documents.upload([Path(directory) / "example.pdf"])

    def test_validation_and_errors(self):
        client = Client("test-key")
        for call in (lambda: client.documents.list(limit=0), lambda: client.documents.create(name=""), lambda: client.documents.update("doc"), lambda: client.workflows.dispatch(event="other", document_id="doc")):
            with self.assertRaises(ValueError):
                call()
        error = HTTPError("https://example.com", 429, "Too Many", {"Retry-After": "10"}, io.BytesIO(b'{"error":"Rate limit reached"}'))
        with patch("formalini.client.urlopen", side_effect=error):
            with self.assertRaises(ApiError) as raised:
                client.documents.list()
        self.assertEqual(raised.exception.status, 429)
        self.assertEqual(raised.exception.retry_after, "10")
        self.assertEqual(str(raised.exception), "Rate limit reached")
        with patch("formalini.client.urlopen", return_value=Response({"wrong": []})):
            with self.assertRaises(MalformedResponse):
                client.documents.list()


if __name__ == "__main__":
    unittest.main()
