#!/bin/sh
redis-benchmark -p 8080 -t SET,GET
