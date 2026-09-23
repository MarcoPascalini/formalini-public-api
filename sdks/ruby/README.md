# Formalini Ruby client

The Ruby client uses `Net::HTTP` and covers the six operations in the [published API docs](https://formalini.com/docs). It has not been published to RubyGems.

```ruby
require 'formalini'
client = Formalini::Client.new(api_key: ENV.fetch('FORMALINI_API_KEY'))
client.documents.list(limit: 10).each { |document| puts document.fetch('id') }
```

The client supports an injected transport for testing, preserves status and `Retry-After` in `ApiError`, and never retries mutations. Uploads assemble multipart bodies in memory; use raw HTTP for very large batches.
