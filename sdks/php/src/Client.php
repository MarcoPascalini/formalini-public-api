<?php

declare(strict_types=1);

namespace Formalini;

use Closure;
use JsonException;
use RuntimeException;

class ApiException extends RuntimeException
{
    public function __construct(public readonly int $status, string $message, public readonly ?string $retryAfter = null)
    {
        parent::__construct($message, $status);
    }
}

final class UsageLimitException extends ApiException
{
}

final class Client
{
    private Closure $transport;

    public function __construct(
        private readonly string $apiKey,
        private readonly string $baseUrl = 'https://api.formalini.com/api/public/v1',
        private readonly int $timeoutSeconds = 60,
        ?Closure $transport = null,
    ) {
        if (trim($apiKey) === '') {
            throw new \InvalidArgumentException('API key is required.');
        }
        if (!str_starts_with($baseUrl, 'https://')) {
            throw new \InvalidArgumentException('Base URL must use HTTPS.');
        }
        if ($timeoutSeconds < 1) {
            throw new \InvalidArgumentException('Timeout must be positive.');
        }
        $this->transport = $transport ?? self::curlTransport(...);
    }

    public function documents(): Documents
    {
        return new Documents($this);
    }

    public function workflows(): Workflows
    {
        return new Workflows($this);
    }

    public function request(string $method, string $path, ?string $body = null, ?string $contentType = null): array
    {
        $headers = ['Authorization: Bearer ' . $this->apiKey];
        if ($contentType !== null) {
            $headers[] = 'Content-Type: ' . $contentType;
        }
        [$status, $raw, $retryAfter] = ($this->transport)(
            $method,
            rtrim($this->baseUrl, '/') . $path,
            $headers,
            $body,
            $this->timeoutSeconds,
        );
        try {
            $result = json_decode($raw, true, 512, JSON_THROW_ON_ERROR);
        } catch (JsonException $error) {
            if ($status >= 400) {
                throw new ApiException($status, "Formalini returned HTTP $status", $retryAfter);
            }
            throw new RuntimeException('Unexpected Formalini response.', previous: $error);
        }
        if (!is_array($result) || array_is_list($result)) {
            throw new RuntimeException('Unexpected Formalini response.');
        }
        if ($status >= 400) {
            $message = is_string($result['error'] ?? null) ? $result['error'] : "Formalini returned HTTP $status";
            $exception = $status === 402 ? UsageLimitException::class : ApiException::class;
            throw new $exception($status, $message, $retryAfter);
        }
        return $result;
    }

    public function json(string $method, string $path, array $payload): array
    {
        return $this->request($method, $path, json_encode($payload, JSON_THROW_ON_ERROR), 'application/json');
    }

    private static function curlTransport(string $method, string $url, array $headers, ?string $body, int $timeout): array
    {
        $curl = curl_init($url);
        if ($curl === false) {
            throw new RuntimeException('Could not initialize HTTP request.');
        }
        $retryAfter = null;
        curl_setopt_array($curl, [
            CURLOPT_CUSTOMREQUEST => $method,
            CURLOPT_HTTPHEADER => $headers,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => $timeout,
            CURLOPT_HEADERFUNCTION => static function ($curl, string $header) use (&$retryAfter): int {
                if (stripos($header, 'Retry-After:') === 0) {
                    $retryAfter = trim(substr($header, strlen('Retry-After:')));
                }
                return strlen($header);
            },
        ]);
        if ($body !== null) {
            curl_setopt($curl, CURLOPT_POSTFIELDS, $body);
        }
        $raw = curl_exec($curl);
        if ($raw === false) {
            $message = curl_error($curl);
            curl_close($curl);
            throw new RuntimeException("Formalini request failed: $message");
        }
        $status = (int) curl_getinfo($curl, CURLINFO_RESPONSE_CODE);
        curl_close($curl);
        return [$status, $raw, $retryAfter];
    }
}

final class Documents
{
    public function __construct(private readonly Client $client)
    {
    }

    public function list(int $limit = 25, ?string $status = null): array
    {
        if ($limit < 1 || $limit > 100) {
            throw new \InvalidArgumentException('Limit must be between 1 and 100.');
        }
        if ($status !== null && !in_array($status, ['queued', 'processing', 'review', 'approved', 'exported', 'failed'], true)) {
            throw new \InvalidArgumentException('Unsupported status.');
        }
        $query = http_build_query(array_filter(['limit' => $limit, 'status' => $status], static fn ($value) => $value !== null));
        $response = $this->client->request('GET', '/documents?' . $query);
        if (!isset($response['documents']) || !is_array($response['documents'])) {
            throw new RuntimeException('Expected a documents array.');
        }
        return $response['documents'];
    }

    public function create(string $name, array $fields = [], array $optional = []): array
    {
        if (trim($name) === '' || count($fields) > 400) {
            throw new \InvalidArgumentException('Name is required and fields are limited to 400.');
        }
        if (array_diff(array_keys($optional), ['category', 'vendor', 'amount', 'document_date'])) {
            throw new \InvalidArgumentException('Unsupported create option.');
        }
        return $this->client->json('POST', '/documents', ['name' => $name, 'fields' => $fields] + $optional);
    }

    public function upload(array $paths, ?string $name = null, ?string $templateId = null, bool $asynchronous = false): array
    {
        if (count($paths) < 1 || count($paths) > 25) {
            throw new \InvalidArgumentException('Upload requires 1 to 25 page images.');
        }
        $boundary = bin2hex(random_bytes(16));
        $body = '';
        $types = ['jpg' => 'image/jpeg', 'jpeg' => 'image/jpeg', 'png' => 'image/png', 'webp' => 'image/webp', 'heic' => 'image/heic'];
        foreach ($paths as $path) {
            $extension = strtolower(pathinfo($path, PATHINFO_EXTENSION));
            if (!isset($types[$extension]) || !is_file($path) || filesize($path) > 100 * 1024 * 1024) {
                throw new \InvalidArgumentException('Each page must be a JPEG, PNG, WebP or HEIC image of at most 100 MB.');
            }
            $filename = str_replace(["\r", "\n", '"'], '_', basename($path));
            $body .= "--$boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"$filename\"\r\nContent-Type: {$types[$extension]}\r\n\r\n";
            $body .= file_get_contents($path) . "\r\n";
        }
        foreach (['name' => $name, 'template_id' => $templateId, 'async' => $asynchronous ? 'true' : null] as $key => $value) {
            if ($value !== null) {
                if (str_contains($value, "\r") || str_contains($value, "\n")) {
                    throw new \InvalidArgumentException('Multipart fields cannot contain newlines.');
                }
                $body .= "--$boundary\r\nContent-Disposition: form-data; name=\"$key\"\r\n\r\n$value\r\n";
            }
        }
        $body .= "--$boundary--\r\n";
        return $this->client->request('POST', '/documents/upload', $body, "multipart/form-data; boundary=$boundary");
    }

    public function get(string $id): array
    {
        if ($id === '') {
            throw new \InvalidArgumentException('Document ID is required.');
        }
        return $this->client->request('GET', '/documents/' . rawurlencode($id));
    }

    public function update(string $id, ?string $status = null, ?array $fields = null): array
    {
        if ($id === '' || ($status === null && $fields === null)) {
            throw new \InvalidArgumentException('Document ID and a change are required.');
        }
        if ($status !== null && !in_array($status, ['review', 'approved', 'exported'], true)) {
            throw new \InvalidArgumentException('Unsupported status.');
        }
        $body = [];
        if ($status !== null) $body['status'] = $status;
        if ($fields !== null) $body['fields'] = $fields;
        return $this->client->json('PATCH', '/documents/' . rawurlencode($id), $body);
    }
}

final class Workflows
{
    public function __construct(private readonly Client $client)
    {
    }

    public function dispatch(string $event, string $documentId, ?array $payload = null): array
    {
        if (!in_array($event, ['document.processed', 'document.needs_review', 'document.approved', 'document.exported'], true) || $documentId === '') {
            throw new \InvalidArgumentException('A documented event and document ID are required.');
        }
        $body = ['event' => $event, 'document_id' => $documentId];
        if ($payload !== null) $body['payload'] = $payload;
        return $this->client->json('POST', '/workflows/dispatch', $body);
    }
}
