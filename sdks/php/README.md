# Formalini PHP client

The PHP 8.2+ client implements the six operations in the [published API documentation](https://formalini.com/docs). It requires cURL and JSON extensions. It is not published to Packagist.

```php
<?php
require 'sdks/php/src/Client.php';

$client = new Formalini\Client(getenv('FORMALINI_API_KEY'));
$documents = $client->documents()->list(limit: 10);
foreach ($documents as $document) {
    echo $document['id'] . PHP_EOL;
}
```

Use `documents()->upload([$imagePath])` for page images. The client preserves server errors through `ApiException` with status and `Retry-After`, and never retries mutations. Large batches are assembled in memory; use raw HTTP for very large uploads. Run `php tests/run.php` from this directory.
