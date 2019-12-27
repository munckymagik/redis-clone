require 'rubygems'
require 'bundler/setup'
Bundler.require(:default)

threads = []

100.times do |i|
  threads.push(Thread.new do
    redis = Redis.new(port: 8080)
    # redis = Redis.new

    1000.times do |j|
      redis.set("#{i}:#{j}:mykey", "hello world #{i}:#{j}")
      redis.get("#{i}:#{j}:mykey")
      redis.del("#{i}:#{j}:mykey")
    end
  end)
end

puts "Waiting for done ..."
threads.each(&:join)
