from __future__ import annotations

import json
import mimetypes
import uuid
from pathlib import Path
from typing import Any, List
from urllib.error import HTTPError
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen


class FormaliniError(Exception):
    pass


class ApiError(FormaliniError):
    def __init__(self, status: int, message: str, retry_after: str | None = None):
        self.status = status
        self.retry_after = retry_after
        super().__init__(message)


class UsageLimitError(ApiError):
    pass


class MalformedResponse(FormaliniError):
    pass


class Client:
    def __init__(
        self,
        api_key: str,
        *,
        base_url: str = "https://api.formalini.com/api/public/v1",
        timeout: float = 60,
    ):
        if not isinstance(api_key, str) or not api_key.strip():
            raise ValueError("api_key must be non-empty")
        if not base_url.startswith("https://"):
            raise ValueError("base_url must use HTTPS")
        if timeout <= 0:
            raise ValueError("timeout must be positive")
        self._api_key = api_key
        self._base_url = base_url.rstrip("/")
        self._timeout = timeout
        self.documents = Documents(self)
        self.workflows = Workflows(self)

    def _request(
        self,
        method: str,
        path: str,
        *,
        body: bytes | None = None,
        content_type: str | None = None,
    ) -> dict[str, Any]:
        headers = {"Authorization": f"Bearer {self._api_key}"}
        if content_type:
            headers["Content-Type"] = content_type
        request = Request(
            self._base_url + path, data=body, headers=headers, method=method
        )
        try:
            with urlopen(request, timeout=self._timeout) as response:
                raw = response.read()
        except HTTPError as error:
            message = f"Formalini returned HTTP {error.code}"
            try:
                payload = json.loads(error.read())
                if isinstance(payload, dict) and isinstance(payload.get("error"), str):
                    message = payload["error"]
            except (ValueError, OSError):
                pass
            error_type = UsageLimitError if error.code == 402 else ApiError
            raise error_type(
                error.code,
                message,
                error.headers.get("Retry-After") if error.headers else None,
            ) from None
        try:
            result = json.loads(raw)
            if not isinstance(result, dict):
                raise ValueError
            return result
        except ValueError as error:
            raise MalformedResponse("Unexpected Formalini response") from error

    def _json(self, method: str, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        return self._request(
            method,
            path,
            body=json.dumps(payload).encode(),
            content_type="application/json",
        )


class Documents:
    def __init__(self, client: Client):
        self._client = client

    def list(
        self, *, limit: int = 25, status: str | None = None
    ) -> List[dict[str, Any]]:
        if (
            isinstance(limit, bool)
            or not isinstance(limit, int)
            or not 1 <= limit <= 100
        ):
            raise ValueError("limit must be an integer from 1 to 100")
        allowed = {"queued", "processing", "review", "approved", "exported", "failed"}
        if status is not None and status not in allowed:
            raise ValueError("unsupported status")
        query: dict[str, str | int] = {"limit": limit}
        if status:
            query["status"] = status
        response = self._client._request("GET", "/documents?" + urlencode(query))
        if not isinstance(response.get("documents"), list):
            raise MalformedResponse("Expected a documents array")
        return response["documents"]

    def create(
        self, *, name: str, fields: List[dict[str, Any]] | None = None, **optional: Any
    ) -> dict[str, Any]:
        if not name or not name.strip():
            raise ValueError("name is required")
        if fields is not None and len(fields) > 400:
            raise ValueError("fields may contain at most 400 entries")
        allowed = {"category", "vendor", "amount", "document_date"}
        if set(optional) - allowed:
            raise ValueError("unsupported create option")
        payload: dict[str, Any] = {"name": name, **optional}
        if fields is not None:
            payload["fields"] = fields
        return self._client._json("POST", "/documents", payload)

    def upload(
        self,
        files: List[str | Path],
        *,
        name: str | None = None,
        template_id: str | None = None,
        asynchronous: bool = False,
    ) -> dict[str, Any]:
        if not 1 <= len(files) <= 25:
            raise ValueError("upload requires 1 to 25 page images")
        boundary = uuid.uuid4().hex
        parts: List[bytes] = []
        for field_value in (name, template_id):
            if field_value is not None and ("\r" in field_value or "\n" in field_value):
                raise ValueError("multipart fields cannot contain newlines")
        for value in files:
            path = Path(value)
            mime = (
                "image/heic"
                if path.suffix.lower() == ".heic"
                else mimetypes.guess_type(path.name)[0]
            )
            if mime not in {"image/jpeg", "image/png", "image/webp", "image/heic"}:
                raise ValueError("upload accepts JPEG, PNG, WebP or HEIC page images")
            if path.stat().st_size > 100 * 1024 * 1024:
                raise ValueError("each page must be at most 100 MB")
            safe_name = (
                path.name.replace('"', "_").replace("\r", "_").replace("\n", "_")
            )
            parts.extend(
                [
                    f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{safe_name}"\r\nContent-Type: {mime}\r\n\r\n'.encode(),
                    path.read_bytes(),
                    b"\r\n",
                ]
            )
        for key, field_value in (
            ("name", name),
            ("template_id", template_id),
            ("async", "true" if asynchronous else None),
        ):
            if field_value is not None:
                parts.append(
                    f'--{boundary}\r\nContent-Disposition: form-data; name="{key}"\r\n\r\n{field_value}\r\n'.encode()
                )
        parts.append(f"--{boundary}--\r\n".encode())
        return self._client._request(
            "POST",
            "/documents/upload",
            body=b"".join(parts),
            content_type=f"multipart/form-data; boundary={boundary}",
        )

    def get(self, document_id: str) -> dict[str, Any]:
        if not document_id:
            raise ValueError("document_id is required")
        return self._client._request("GET", "/documents/" + quote(document_id, safe=""))

    def update(
        self,
        document_id: str,
        *,
        status: str | None = None,
        fields: List[dict[str, Any]] | None = None,
    ) -> dict[str, Any]:
        if not document_id or (status is None and fields is None):
            raise ValueError("document_id and a change are required")
        if status is not None and status not in {"review", "approved", "exported"}:
            raise ValueError("unsupported update status")
        payload: dict[str, Any] = {}
        if status is not None:
            payload["status"] = status
        if fields is not None:
            payload["fields"] = fields
        return self._client._json(
            "PATCH", "/documents/" + quote(document_id, safe=""), payload
        )


class Workflows:
    def __init__(self, client: Client):
        self._client = client

    def dispatch(
        self, *, event: str, document_id: str, payload: dict[str, Any] | None = None
    ) -> dict[str, Any]:
        if event not in {
            "document.processed",
            "document.needs_review",
            "document.approved",
            "document.exported",
        }:
            raise ValueError("unsupported event")
        if not document_id:
            raise ValueError("document_id is required")
        body: dict[str, Any] = {"event": event, "document_id": document_id}
        if payload is not None:
            body["payload"] = payload
        return self._client._json("POST", "/workflows/dispatch", body)
