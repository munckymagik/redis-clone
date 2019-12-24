require 'rubygems'
require 'bundler/setup'
Bundler.require(:default)

require 'minitest/autorun'

SUPPORTED_COMMANDS = ['set', 'get', 'del', 'command'].freeze

class CommandTest < Minitest::Test
  def test_unknown_command
    real_unknown_cmd_msg = self.capture_error(real_redis, :xyz, [1, 2, 3])
    clone_unknown_cmd_msg = self.capture_error(clone_redis, :xyz, [1, 2, 3])
    assert_equal real_unknown_cmd_msg, clone_unknown_cmd_msg

    real_unknown_cmd_msg = self.capture_error(real_redis, :abc, [])
    clone_unknown_cmd_msg = self.capture_error(clone_redis, :abc, [])
    assert_equal real_unknown_cmd_msg, clone_unknown_cmd_msg
  end

  def test_supported_commands
    all_real_commands = real_redis.command
    real_supported = all_real_commands.select { |c| SUPPORTED_COMMANDS.include? c.first }
    # real_supported.each { |o| puts o.inspect }

    all_clone_commands = clone_redis.command

    assert_equal all_real_commands, all_clone_commands
  end

  def teardown
    real_redis.close
    clone_redis.close
  end

  def real_redis
    @real_redis ||= Redis.new
  end

  def clone_redis
    @clone_redis ||= Redis.new(port: 8080)
  end

  def capture_error(server, cmd, args)
    server.send(cmd, args)
  rescue => e
    e.message
  end
end
