RSpec.shared_context "Redis connection" do
  let(:redis) { Redis.new(port: 8080) }

  around(:example) do |example|
    example.run
  ensure
    redis.flushdb
  end
end

RSpec.configure do |rspec|
  rspec.include_context "Redis connection", :include_connection => true
end
