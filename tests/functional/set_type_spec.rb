RSpec.describe "Set commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "sadd").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "scard").dig(0, 1)).to eql(2)
      expect(redis.command("info", "smembers").dig(0, 1)).to eql(2)
      expect(redis.command("info", "srem").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "sscan").dig(0, 1)).to eql(-3)
    end
  end

  describe "commands used against the wrong type" do
    let(:expected_error) { "WRONGTYPE Operation against a key holding the wrong kind of value" }

    specify "raise an error" do
      redis.set("x", "not a set")

      expect { redis.sadd("x", "y") }
        .to raise_error(expected_error)
      expect { redis.scard("x") }
        .to raise_error(expected_error)
      expect { redis.smembers("x") }
        .to raise_error(expected_error)
      expect { redis.srem("x", "y") }
        .to raise_error(expected_error)
      expect { redis.sscan("x", "0") }
        .to raise_error(expected_error)
    end
  end
end
