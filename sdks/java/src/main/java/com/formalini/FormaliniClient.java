package com.formalini;

import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

public final class FormaliniClient {
    private static final Gson JSON = new Gson();
    private final HttpClient http;
    private final String apiKey;
    private final URI baseUrl;
    private final Duration timeout;

    public final Documents documents = new Documents();
    public final Workflows workflows = new Workflows();

    public FormaliniClient(String apiKey) {
        this(apiKey, URI.create("https://api.formalini.com/api/public/v1/"), HttpClient.newHttpClient(), Duration.ofSeconds(60));
    }

    public FormaliniClient(String apiKey, URI baseUrl, HttpClient http, Duration timeout) {
        if (apiKey == null || apiKey.isBlank()) throw new IllegalArgumentException("API key is required");
        if (!"https".equals(baseUrl.getScheme())) throw new IllegalArgumentException("Base URL must use HTTPS");
        if (timeout.isZero() || timeout.isNegative()) throw new IllegalArgumentException("Timeout must be positive");
        this.apiKey = apiKey;
        this.baseUrl = baseUrl.toString().endsWith("/") ? baseUrl : URI.create(baseUrl + "/");
        this.http = http;
        this.timeout = timeout;
    }

    private JsonObject send(String method, String path, byte[] body, String contentType) throws IOException, InterruptedException {
        HttpRequest.Builder request = HttpRequest.newBuilder(baseUrl.resolve(path))
            .timeout(timeout)
            .header("Authorization", "Bearer " + apiKey);
        if (contentType != null) request.header("Content-Type", contentType);
        request.method(method, body == null ? HttpRequest.BodyPublishers.noBody() : HttpRequest.BodyPublishers.ofByteArray(body));
        HttpResponse<String> response = http.send(request.build(), HttpResponse.BodyHandlers.ofString());
        JsonObject result;
        try {
            result = JsonParser.parseString(response.body()).getAsJsonObject();
        } catch (RuntimeException error) {
            if (response.statusCode() >= 400) throw new ApiException(response.statusCode(), "Formalini returned HTTP " + response.statusCode(), null);
            throw new IOException("Unexpected Formalini response", error);
        }
        if (response.statusCode() >= 400) {
            String message = result.has("error") && result.get("error").isJsonPrimitive()
                ? result.get("error").getAsString() : "Formalini returned HTTP " + response.statusCode();
            throw new ApiException(response.statusCode(), message, response.headers().firstValue("Retry-After").orElse(null));
        }
        return result;
    }

    private JsonObject json(String method, String path, Object payload) throws IOException, InterruptedException {
        return send(method, path, JSON.toJson(payload).getBytes(StandardCharsets.UTF_8), "application/json");
    }

    public final class Documents {
        public List<com.google.gson.JsonElement> list(int limit, String status) throws IOException, InterruptedException {
            if (limit < 1 || limit > 100) throw new IllegalArgumentException("Limit must be between 1 and 100");
            if (status != null && !Set.of("queued", "processing", "review", "approved", "exported", "failed").contains(status))
                throw new IllegalArgumentException("Unsupported status");
            String path = "documents?limit=" + limit + (status == null ? "" : "&status=" + URLEncoder.encode(status, StandardCharsets.UTF_8));
            JsonObject response = send("GET", path, null, null);
            if (!response.has("documents") || !response.get("documents").isJsonArray()) throw new IOException("Expected a documents array");
            return response.getAsJsonArray("documents").asList();
        }

        public JsonObject create(String name, List<Map<String, Object>> fields, Map<String, Object> optional) throws IOException, InterruptedException {
            if (name == null || name.isBlank()) throw new IllegalArgumentException("Name is required");
            if (fields != null && fields.size() > 400) throw new IllegalArgumentException("Too many fields");
            if (optional.keySet().stream().anyMatch(key -> !Set.of("category", "vendor", "amount", "document_date").contains(key)))
                throw new IllegalArgumentException("Unsupported create option");
            JsonObject payload = new JsonObject();
            payload.addProperty("name", name);
            if (fields != null) payload.add("fields", JSON.toJsonTree(fields));
            optional.forEach((key, value) -> payload.add(key, JSON.toJsonTree(value)));
            return json("POST", "documents", payload);
        }

        public JsonObject upload(List<Path> images, String name, String templateId, boolean asynchronous) throws IOException, InterruptedException {
            if (images == null || images.isEmpty() || images.size() > 25) throw new IllegalArgumentException("Upload requires 1 to 25 page images");
            String boundary = UUID.randomUUID().toString();
            ByteArrayOutputStream body = new ByteArrayOutputStream();
            for (Path image : images) {
                String filename = image.getFileName().toString().replaceAll("[\"\\r\\n]", "_");
                String lower = filename.toLowerCase();
                String mime = lower.endsWith(".jpg") || lower.endsWith(".jpeg") ? "image/jpeg" :
                    lower.endsWith(".png") ? "image/png" : lower.endsWith(".webp") ? "image/webp" :
                    lower.endsWith(".heic") ? "image/heic" : null;
                if (mime == null || Files.size(image) > 100L * 1024 * 1024) throw new IllegalArgumentException("Unsupported or oversized page image");
                write(body, "--" + boundary + "\r\nContent-Disposition: form-data; name=\"file\"; filename=\"" + filename + "\"\r\nContent-Type: " + mime + "\r\n\r\n");
                Files.copy(image, body);
                write(body, "\r\n");
            }
            if (name != null) part(body, boundary, "name", name);
            if (templateId != null) part(body, boundary, "template_id", templateId);
            if (asynchronous) part(body, boundary, "async", "true");
            write(body, "--" + boundary + "--\r\n");
            return send("POST", "documents/upload", body.toByteArray(), "multipart/form-data; boundary=" + boundary);
        }

        public JsonObject get(String id) throws IOException, InterruptedException {
            return send("GET", documentPath(id), null, null);
        }

        public JsonObject update(String id, String status, List<Map<String, Object>> fields) throws IOException, InterruptedException {
            if (status == null && fields == null) throw new IllegalArgumentException("A change is required");
            if (status != null && !Set.of("review", "approved", "exported").contains(status)) throw new IllegalArgumentException("Unsupported status");
            JsonObject payload = new JsonObject();
            if (status != null) payload.addProperty("status", status);
            if (fields != null) payload.add("fields", JSON.toJsonTree(fields));
            return json("PATCH", documentPath(id), payload);
        }

        private String documentPath(String id) {
            if (id == null || id.isBlank()) throw new IllegalArgumentException("Document ID is required");
            return "documents/" + URLEncoder.encode(id, StandardCharsets.UTF_8);
        }
    }

    public final class Workflows {
        public JsonObject dispatch(String event, String documentId, Map<String, Object> payload) throws IOException, InterruptedException {
            if (!Set.of("document.processed", "document.needs_review", "document.approved", "document.exported").contains(event)
                || documentId == null || documentId.isBlank()) throw new IllegalArgumentException("A documented event and document ID are required");
            JsonObject body = new JsonObject();
            body.addProperty("event", event);
            body.addProperty("document_id", documentId);
            if (payload != null) body.add("payload", JSON.toJsonTree(payload));
            return json("POST", "workflows/dispatch", body);
        }
    }

    private static void part(ByteArrayOutputStream body, String boundary, String key, String value) throws IOException {
        if (value.contains("\r") || value.contains("\n")) throw new IllegalArgumentException("Multipart fields cannot contain newlines");
        write(body, "--" + boundary + "\r\nContent-Disposition: form-data; name=\"" + key + "\"\r\n\r\n" + value + "\r\n");
    }

    private static void write(ByteArrayOutputStream body, String text) throws IOException {
        body.write(text.getBytes(StandardCharsets.UTF_8));
    }

    public static final class ApiException extends IOException {
        private final int status;
        private final String retryAfter;

        public ApiException(int status, String message, String retryAfter) {
            super(message);
            this.status = status;
            this.retryAfter = retryAfter;
        }

        public int status() { return status; }
        public String retryAfter() { return retryAfter; }
    }
}
