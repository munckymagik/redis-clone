def using_real_redis?
  ENV.key? 'TEST_REAL_REDIS'
end

def redis_5_or_older?
  redis.info("server")["redis_version"].split(".")[0].to_i <= 5
end

RSpec.shared_context "Redis connection" do
  let(:port) do
    if using_real_redis?
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

  if using_real_redis?
    rspec.filter_run_excluding redis_clone_only: true

    rspec.after(:suite) do
      puts
      puts "XXXX -------------------------------------------------------------"
      puts "XXXX tests were ran against the real Redis"
      puts "XXXX -------------------------------------------------------------"
    end
  end
end
