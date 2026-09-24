require 'minitest/autorun'
require 'tempfile'
require_relative '../lib/formalini'

class ClientTest < Minitest::Test
  def setup
    @calls = []
    calls = @calls
    transport = ->(method, url, headers, body, timeout) {
      calls << { method: method, url: url, headers: headers, body: body, timeout: timeout }
      [200, '{"documents":[{"id":"example"}],"ok":true}', nil]
    }
    @client = Formalini::Client.new(api_key: 'test-key', transport: transport)
  end

  def test_all_requests
    assert_equal 'example', @client.documents.list(limit: 10, status: 'approved').first.fetch('id')
    assert_equal 'https://api.formalini.com/api/public/v1/documents?limit=10&status=approved', @calls[0][:url]
    assert_equal 'Bearer test-key', @calls[0][:headers]['Authorization']

    @client.documents.create(name: 'Invoice', fields: [])
    assert_equal 'POST', @calls[1][:method]
    assert_equal 'Invoice', JSON.parse(@calls[1][:body]).fetch('name')

    @client.documents.get('doc-id')
    assert @calls[2][:url].end_with?('/documents/doc-id')
    @client.documents.update('doc-id', status: 'approved')
    assert_equal 'PATCH', @calls[3][:method]
    @client.workflows.dispatch(event: 'document.approved', document_id: 'doc-id')
    assert_equal 'doc-id', JSON.parse(@calls[4][:body]).fetch('document_id')

    Tempfile.create(['page', '.jpg']) do |file|
      file.write('image fixture')
      file.flush
      @client.documents.upload(paths: [file.path], asynchronous: true)
    end
    assert_includes @calls[5][:body], 'name="file"'
    assert_includes @calls[5][:body], 'name="async"'
  end

  def test_validation_and_errors
    assert_raises(ArgumentError) { @client.documents.list(limit: 0) }
    assert_raises(ArgumentError) { @client.documents.upload(paths: ['file.pdf']) }
    denied = Formalini::Client.new(api_key: 'test-key', transport: ->(*) { [403, '{"error":"Insufficient scope."}', nil] })
    error = assert_raises(Formalini::ApiError) { denied.documents.get('doc-id') }
    assert_equal 403, error.status
    assert_equal 'Insufficient scope.', error.message
    limited = Formalini::Client.new(api_key: 'test-key', transport: ->(*) { [402, '{"error":"No credits remaining"}', nil] })
    assert_raises(Formalini::UsageLimitError) { limited.documents.get('doc-id') }
  end
end
