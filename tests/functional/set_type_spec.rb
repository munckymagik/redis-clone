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
      # expect { redis.sscan("x", "0") }
      #   .to raise_error(expected_error)
    end
  end

  describe "SADD" do
    context "when the db key does not already exist" do
      it "creates one" do
        expect(redis.sadd("x", "y")).to be(true)
        expect(redis.type("x")).to eql("set")
      end
    end

    context "when the db key already exists" do
      it "adds only new values to the set" do
        expect(redis.sadd("x", %w[w])).to eql(1)
        expect(redis.scard("x")).to eql(1)

        expect(redis.sadd("x", %w[y z])).to eq(2)
        expect(redis.smembers("x")).to contain_exactly(*%w[w y z])

        expect(redis.sadd("x", %w[y zz])).to eq(1)
        expect(redis.smembers("x")).to contain_exactly(*%w[w y z zz])
      end
    end
  end

  describe "SCARD" do
    context "when the key does not already exist" do
      it "returns 0" do
        expect(redis.scard("x")).to eql(0)
      end
    end

    context "when the key does exist" do
      it "returns the size" do
        redis.sadd("x", [1, 2])
        expect(redis.scard("x")).to eql(2)
        redis.sadd("x", 3)
        expect(redis.scard("x")).to eql(3)
      end
    end
  end

  describe "SMEMBERS" do
    context "when the db key does not already exist" do
      it "returns an empty array" do
        expect(redis.smembers("x")).to eql([])
      end
    end

    context "when the db key already exists" do
      it "returns an array of values" do
        redis.sadd("x", %w[y z])
        expect(redis.smembers("x")).to contain_exactly(*%w[y z])
      end
    end
  end

  describe "SREM" do
    context "when the key does not exist" do
      it "returns zero" do
        expect(redis.srem("x", "blah")).to be(false)
      end
    end

    context "when the key exists" do
      before do
          redis.sadd("myset", %w[x y z])
      end

      specify "removing all values" do
        expect(redis.srem("myset", %w[x y z])).to eql(3)
        expect(redis.exists("myset")).to be(false)
      end

      specify "removing some of the values" do
        expect(redis.srem("myset", %w[x y])).to eql(2)
        expect(redis.smembers("myset")).to contain_exactly(*%w[z])
      end

      specify "trying to remove a non-existent element" do
        expect(redis.srem("myset", %w[a z])).to eql(1)
        expect(redis.smembers("myset")).to contain_exactly(*%w[x y])
      end
    end
  end
end
