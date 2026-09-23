require 'json'
require 'net/http'
require 'securerandom'
require 'uri'

module Formalini
  class ApiError < StandardError
    attr_reader :status, :retry_after

    def initialize(status, message, retry_after = nil)
      @status = status
      @retry_after = retry_after
      super(message)
    end
  end

  class Client
    attr_reader :documents, :workflows

    def initialize(api_key:, base_url: 'https://api.formalini.com/api/public/v1', timeout: 60, transport: nil)
      raise ArgumentError, 'API key is required' if api_key.to_s.strip.empty?
      raise ArgumentError, 'Base URL must use HTTPS' unless base_url.start_with?('https://')
      raise ArgumentError, 'Timeout must be positive' unless timeout.positive?

      @api_key = api_key
      @base_url = base_url.delete_suffix('/')
      @timeout = timeout
      @transport = transport || method(:send_http)
      @documents = Documents.new(self)
      @workflows = Workflows.new(self)
    end

    def request(method, path, body: nil, content_type: nil)
      headers = { 'Authorization' => "Bearer #{@api_key}" }
      headers['Content-Type'] = content_type if content_type
      status, raw, retry_after = @transport.call(method, "#{@base_url}#{path}", headers, body, @timeout)
      begin
        result = JSON.parse(raw)
      rescue JSON::ParserError
        raise ApiError.new(status, "Formalini returned HTTP #{status}", retry_after) if status >= 400
        raise 'Unexpected Formalini response'
      end
      raise 'Unexpected Formalini response' unless result.is_a?(Hash)

      if status >= 400
        message = result['error'].is_a?(String) ? result['error'] : "Formalini returned HTTP #{status}"
        raise ApiError.new(status, message, retry_after)
      end
      result
    end

    def json(method, path, payload)
      request(method, path, body: JSON.generate(payload), content_type: 'application/json')
    end

    private

    def send_http(method, url, headers, body, timeout)
      uri = URI(url)
      http = Net::HTTP.new(uri.host, uri.port)
      http.use_ssl = true
      http.open_timeout = timeout
      http.read_timeout = timeout
      request = Net::HTTPGenericRequest.new(method, body.nil?, !body.nil?, uri.request_uri, headers)
      request.body = body if body
      response = http.request(request)
      [response.code.to_i, response.body, response['Retry-After']]
    end
  end

  class Documents
    STATUSES = %w[queued processing review approved exported failed].freeze
    IMAGE_TYPES = { '.jpg' => 'image/jpeg', '.jpeg' => 'image/jpeg', '.png' => 'image/png', '.webp' => 'image/webp', '.heic' => 'image/heic' }.freeze

    def initialize(client)
      @client = client
    end

    def list(limit: 25, status: nil)
      raise ArgumentError, 'Limit must be between 1 and 100' unless limit.is_a?(Integer) && (1..100).cover?(limit)
      raise ArgumentError, 'Unsupported status' if status && !STATUSES.include?(status)

      query = URI.encode_www_form({ limit: limit, status: status }.compact)
      result = @client.request('GET', "/documents?#{query}")
      raise 'Expected a documents array' unless result['documents'].is_a?(Array)

      result['documents']
    end

    def create(name:, fields: nil, **optional)
      raise ArgumentError, 'Name is required' if name.to_s.strip.empty?
      raise ArgumentError, 'Too many fields' if fields && fields.length > 400
      raise ArgumentError, 'Unsupported create option' unless (optional.keys - %i[category vendor amount document_date]).empty?

      payload = { name: name, **optional }
      payload[:fields] = fields unless fields.nil?
      @client.json('POST', '/documents', payload)
    end

    def upload(paths:, name: nil, template_id: nil, asynchronous: false)
      raise ArgumentError, 'Upload requires 1 to 25 page images' unless (1..25).cover?(paths.length)

      boundary = SecureRandom.hex(16)
      body = +''
      paths.each do |path|
        mime = IMAGE_TYPES[File.extname(path).downcase]
        raise ArgumentError, 'Unsupported or oversized page image' unless mime && File.file?(path) && File.size(path) <= 100 * 1024 * 1024

        filename = File.basename(path).tr("\r\n\"", '___')
        body << "--#{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"#{filename}\"\r\nContent-Type: #{mime}\r\n\r\n"
        body << File.binread(path) << "\r\n"
      end
      { name: name, template_id: template_id, async: (asynchronous ? 'true' : nil) }.compact.each do |key, value|
        raise ArgumentError, 'Multipart fields cannot contain newlines' if value.include?("\r") || value.include?("\n")
        body << "--#{boundary}\r\nContent-Disposition: form-data; name=\"#{key}\"\r\n\r\n#{value}\r\n"
      end
      body << "--#{boundary}--\r\n"
      @client.request('POST', '/documents/upload', body: body, content_type: "multipart/form-data; boundary=#{boundary}")
    end

    def get(id)
      raise ArgumentError, 'Document ID is required' if id.to_s.empty?
      @client.request('GET', "/documents/#{URI.encode_www_form_component(id)}")
    end

    def update(id, status: nil, fields: nil)
      raise ArgumentError, 'Document ID and a change are required' if id.to_s.empty? || (status.nil? && fields.nil?)
      raise ArgumentError, 'Unsupported status' if status && !%w[review approved exported].include?(status)

      payload = {}
      payload[:status] = status if status
      payload[:fields] = fields unless fields.nil?
      @client.json('PATCH', "/documents/#{URI.encode_www_form_component(id)}", payload)
    end
  end

  class Workflows
    EVENTS = %w[document.processed document.needs_review document.approved document.exported].freeze

    def initialize(client)
      @client = client
    end

    def dispatch(event:, document_id:, payload: nil)
      raise ArgumentError, 'A documented event and document ID are required' unless EVENTS.include?(event) && !document_id.to_s.empty?

      body = { event: event, document_id: document_id }
      body[:payload] = payload unless payload.nil?
      @client.json('POST', '/workflows/dispatch', body)
    end
  end
end
