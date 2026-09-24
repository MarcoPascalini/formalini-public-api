<?php

declare(strict_types=1);

require dirname(__DIR__) . '/src/Client.php';

use Formalini\ApiException;
use Formalini\Client;
use Formalini\UsageLimitException;

function check(bool $condition, string $label): void
{
    if (!$condition) throw new RuntimeException("Failed: $label");
}

$calls = [];
$transport = function (string $method, string $url, array $headers, ?string $body, int $timeout) use (&$calls): array {
    $calls[] = compact('method', 'url', 'headers', 'body', 'timeout');
    return [200, '{"documents":[{"id":"example"}],"ok":true}', null];
};
$client = new Client('test-key', transport: $transport);

check($client->documents()->list(limit: 10, status: 'approved')[0]['id'] === 'example', 'List response');
check(str_ends_with($calls[0]['url'], '/documents?limit=10&status=approved'), 'List URL');
check(in_array('Authorization: Bearer test-key', $calls[0]['headers'], true), 'Bearer auth');

$client->documents()->create('Invoice', [['name' => 'total', 'value' => '4']]);
check($calls[1]['method'] === 'POST' && json_decode($calls[1]['body'], true)['name'] === 'Invoice', 'Create request');
$client->documents()->get('doc-id');
check(str_ends_with($calls[2]['url'], '/documents/doc-id'), 'Get request');
$client->documents()->update('doc-id', status: 'approved');
check($calls[3]['method'] === 'PATCH' && json_decode($calls[3]['body'], true)['status'] === 'approved', 'Update request');
$client->workflows()->dispatch('document.approved', 'doc-id');
check(json_decode($calls[4]['body'], true)['document_id'] === 'doc-id', 'Dispatch request');

$image = tempnam(sys_get_temp_dir(), 'formalini-') . '.jpg';
file_put_contents($image, 'image fixture');
try {
    $client->documents()->upload([$image], asynchronous: true);
    check(str_contains($calls[5]['body'], 'name="file"'), 'Upload file field');
    check(str_contains($calls[5]['body'], 'name="async"'), 'Upload async field');
} finally {
    unlink($image);
}

try {
    $client->documents()->list(limit: 0);
    throw new RuntimeException('Expected limit failure');
} catch (InvalidArgumentException) {
}

$denied = new Client('test-key', transport: static fn () => [403, '{"error":"Insufficient scope."}', null]);
try {
    $denied->documents()->get('doc-id');
    throw new RuntimeException('Expected API error');
} catch (ApiException $error) {
    check($error->status === 403 && $error->getMessage() === 'Insufficient scope.', 'API error mapping');
}

$limited = new Client('test-key', transport: static fn () => [402, '{"error":"No credits remaining"}', null]);
try {
    $limited->documents()->get('doc-id');
    throw new RuntimeException('Expected usage-limit error');
} catch (UsageLimitException $error) {
    check($error->status === 402, 'Usage-limit mapping');
}

echo "Formalini PHP request tests passed.\n";
