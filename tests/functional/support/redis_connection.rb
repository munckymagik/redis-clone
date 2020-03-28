def using_real_redis?
  ENV.key? 'TEST_REAL_REDIS'
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
  else
    redis_pid = nil
    rspec.before(:suite) do
      redis_pid = Process.spawn('../../target/release/redis-clone')
      puts "---------------- Spawned #{redis_pid}"

      3.times do |i|
        begin
          puts "-------------- Connection attempt #{i}"
          Redis.new(port: 8080)
          puts "-------------- Got a connection"
          break
        rescue
          raise if i == 2
          sleep 0.2
        end
      end
    end
    rspec.after(:suite) do
      puts "---------------- Killing #{redis_pid}"
      Process.kill("TERM", redis_pid)
      puts "---------------- Waiting on #{redis_pid}"
      Process.wait(redis_pid)
      puts "---------------- Done."
    end
  end

end
