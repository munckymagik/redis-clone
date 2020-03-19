#!/bin/sh

benches="set,get,incr,lpush,rpush,lpop,rpop,lrange,hset"
opts="-q"

redis-cli flushdb
redis-cli -p 8080 flushdb

echo "Real Redis"
echo "------------------------------------------------------------------------"
redis-benchmark $opts -t $benches

echo "Clone Redis"
echo "------------------------------------------------------------------------"
redis-benchmark $opts -t $benches -p 8080
