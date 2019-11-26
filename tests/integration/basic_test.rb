require 'rubygems'
require 'bundler/setup'
Bundler.require(:default)

redis = Redis.new(port: 8080)

puts redis.set("mykey", "hello world")
puts redis.get("mykey")
puts redis.del("mykey")
