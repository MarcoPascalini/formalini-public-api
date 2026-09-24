package com.formalini;

import java.io.IOException;
import java.net.Authenticator;
import java.net.CookieHandler;
import java.net.ProxySelector;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpHeaders;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Executor;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLParameters;

public final class FormaliniClientTest {
    public static void main(String[] args) throws Exception {
        FakeHttp http = new FakeHttp();
        FormaliniClient client = new FormaliniClient("test-key", URI.create("https://api.formalini.com/api/public/v1/"), http, Duration.ofSeconds(5));
        check(client.documents.list(10, "approved").size() == 1, "list response");
        check(http.last.uri().toString().endsWith("/documents?limit=10&status=approved"), "list URL");
        check(http.last.headers().firstValue("Authorization").orElse("").equals("Bearer test-key"), "bearer auth");

        client.documents.create("Invoice", List.of(), Map.of());
        check(http.last.method().equals("POST") && http.last.uri().getPath().endsWith("/documents"), "create request");
        client.documents.get("doc-id");
        check(http.last.uri().getPath().endsWith("/documents/doc-id"), "get request");
        client.documents.update("doc-id", "approved", null);
        check(http.last.method().equals("PATCH"), "update method");
        client.workflows.dispatch("document.approved", "doc-id", Map.of());
        check(http.last.uri().getPath().endsWith("/workflows/dispatch"), "dispatch request");

        Path image = Files.createTempFile("formalini-test", ".jpg");
        try {
            Files.write(image, new byte[] {1, 2, 3});
            client.documents.upload(List.of(image), null, null, true);
            check(http.last.headers().firstValue("Content-Type").orElse("").startsWith("multipart/form-data"), "upload multipart");
        } finally {
            Files.delete(image);
        }

        http.status = 403;
        http.body = "{\"error\":\"Insufficient scope.\"}";
        try {
            client.documents.get("doc-id");
            throw new AssertionError("expected API error");
        } catch (FormaliniClient.ApiException error) {
            check(error.status() == 403 && error.getMessage().equals("Insufficient scope."), "error mapping");
        }
        http.status = 402;
        http.body = "{\"error\":\"No credits remaining\"}";
        try {
            client.documents.get("doc-id");
            throw new AssertionError("expected usage-limit error");
        } catch (FormaliniClient.UsageLimitException error) {
            check(error.status() == 402, "usage-limit mapping");
        }
        System.out.println("Formalini Java request tests passed.");
    }

    private static void check(boolean condition, String label) {
        if (!condition) throw new AssertionError(label);
    }

    private static final class FakeHttp extends HttpClient {
        HttpRequest last;
        int status = 200;
        String body = "{\"documents\":[{\"id\":\"example\"}]}";

        @Override public Optional<CookieHandler> cookieHandler() { return Optional.empty(); }
        @Override public Optional<Duration> connectTimeout() { return Optional.empty(); }
        @Override public Redirect followRedirects() { return Redirect.NEVER; }
        @Override public Optional<ProxySelector> proxy() { return Optional.empty(); }
        @Override public SSLContext sslContext() { return null; }
        @Override public SSLParameters sslParameters() { return new SSLParameters(); }
        @Override public Optional<Authenticator> authenticator() { return Optional.empty(); }
        @Override public Version version() { return Version.HTTP_1_1; }
        @Override public Optional<Executor> executor() { return Optional.empty(); }
        @Override public <T> HttpResponse<T> send(HttpRequest request, HttpResponse.BodyHandler<T> handler) throws IOException {
            last = request;
            @SuppressWarnings("unchecked") HttpResponse<T> response = (HttpResponse<T>) new FakeResponse(request, status, body);
            return response;
        }
        @Override public <T> CompletableFuture<HttpResponse<T>> sendAsync(HttpRequest request, HttpResponse.BodyHandler<T> handler) { throw new UnsupportedOperationException(); }
        @Override public <T> CompletableFuture<HttpResponse<T>> sendAsync(HttpRequest request, HttpResponse.BodyHandler<T> handler, HttpResponse.PushPromiseHandler<T> pushPromiseHandler) { throw new UnsupportedOperationException(); }
    }

    private static final class FakeResponse implements HttpResponse<String> {
        private final HttpRequest request;
        private final int status;
        private final String body;
        FakeResponse(HttpRequest request, int status, String body) { this.request = request; this.status = status; this.body = body; }
        @Override public int statusCode() { return status; }
        @Override public HttpRequest request() { return request; }
        @Override public Optional<HttpResponse<String>> previousResponse() { return Optional.empty(); }
        @Override public HttpHeaders headers() { return HttpHeaders.of(Map.of(), (a, b) -> true); }
        @Override public String body() { return body; }
        @Override public Optional<javax.net.ssl.SSLSession> sslSession() { return Optional.empty(); }
        @Override public URI uri() { return request.uri(); }
        @Override public HttpClient.Version version() { return HttpClient.Version.HTTP_1_1; }
    }
}
