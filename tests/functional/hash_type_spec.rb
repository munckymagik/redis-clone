RSpec.describe "Hash commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "hset").dig(0, 1)).to eql(-4)
      expect(redis.command("info", "hget").dig(0, 1)).to eql(3)
      expect(redis.command("info", "hmset").dig(0, 1)).to eql(-4)
      expect(redis.command("info", "hmget").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "hgetall").dig(0, 1)).to eql(2)
    end
  end

  describe "commands used against the wrong type" do
    let(:expected_error) { "WRONGTYPE Operation against a key holding the wrong kind of value" }

    specify "raise an error" do
      redis.set("x", "not a hash")

      expect { redis.hset("x", "y", "z") }
        .to raise_error(expected_error)
      expect { redis.hget("x", "y") }
        .to raise_error(expected_error)
      expect { redis.hmset("x", "y", "z") }
        .to raise_error(expected_error)
      expect { redis.hmget("x", "y") }
        .to raise_error(expected_error)
      expect { redis.hgetall("x") }
        .to raise_error(expected_error)
    end
  end
end
