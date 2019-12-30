RSpec.shared_context "Redis connection" do
  let(:port) do
    if ENV.key? 'TEST_REAL_REDIS'
      6379
    else
      8080
    end
  end
  let(:redis) { Redis.new(port: port) }

  around(:example) do |example|
    example.run
  ensure
    redis.flushdb
  end
end

RSpec.configure do |rspec|
  rspec.include_context "Redis connection", :include_connection => true

  rspec.after(:suite) do
    if ENV.key? 'TEST_REAL_REDIS'
      puts
      puts "XXXX -------------------------------------------------------------"
      puts "XXXX tests were ran against the real Redis"
      puts "XXXX -------------------------------------------------------------"
    end
  end
end
