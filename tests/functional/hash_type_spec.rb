RSpec.describe "Hash commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "hset").dig(0, 1)).to eql(-4)
      expect(redis.command("info", "hmset").dig(0, 1)).to eql(-4)
      expect(redis.command("info", "hget").dig(0, 1)).to eql(3)
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

  describe "HSET" do
    context "when the db key does not already exist" do
      it "creates one" do
        expect(redis.hset("x", "y", "z")).to be(true)
        expect(redis.type("x")).to eql("hash")
      end
    end

    context "when the db key already exists" do
      it "overwrites the value" do
        redis.hset("x", "y", "z")
        expect(redis.hget("x", "y")).to eql("z")
        redis.hset("x", "y", "zz")
        expect(redis.hget("x", "y")).to eql("zz")
      end
    end

    describe "variable arguments" do
      context "when only new keys are being added" do
        it "adds the keys and returns the count of new keys added" do
          # Note: using `call` because the Ruby client doesn't seem to support
          # variadic HSET
          expect(redis.call("hset", "x", "a", 1, "b", 2)).to eql(2)

          expect(redis.hget("x", "a")).to eql("1")
          expect(redis.hget("x", "b")).to eql("2")
        end
      end

      context "when only some new keys are being added and others being updated" do
        it "adds the new keys, updates the existing and returns the count of new keys added" do
          redis.hset("x", "a", 1)
          # Note: using `call` because the Ruby client doesn't seem to support
          # variadic HSET
          expect(redis.call("hset", "x", "a", 2, "b", 2)).to eql(1)

          expect(redis.hget("x", "a")).to eql("2")
          expect(redis.hget("x", "b")).to eql("2")
        end
      end

      context "when there is an uneven number of arguments" do
        it "returns and error" do
          expect {
            redis.call("hset", "x", "a", 2, "b")
          }.to raise_error("ERR wrong number of arguments for HMSET")
        end
      end
    end
  end

  describe "HMSET" do
    it "is identical to HSET except it has a different return value" do
      expect(redis.hmset("x", "a", 1, "b", 2)).to eql("OK")
      expect(redis.hmset("x", "a", 1, "c", 2)).to eql("OK")
    end
  end

  describe "HGET" do
    context "when the db key does not already exist" do
      it "returns null" do
        expect(redis.hget("x", "y")).to be_nil
      end
    end

    context "when the db key already exists" do
      it "returns the value at that key" do
        # When the value is a string
        redis.hset("x", "y", "z")
        expect(redis.hget("x", "y")).to eql("z")

        # When the value is an int
        redis.hset("x", "y", "1")
        expect(redis.hget("x", "y")).to eql("1")
      end
    end

    context "when the db key already exists, but the hash key does not" do
      it "returns nil" do
        # When the value is a string
        redis.hset("x", "y", "z")
        expect(redis.hget("x", "z")).to be_nil
      end
    end
  end

  describe "HGETALL" do
    context "when the db key does not already exist" do
      it "returns an empty array" do
        # Underneath the client library, Redis returns an array
        expect(redis.call("hgetall", "x")).to eql([])
        # The client library knows to chunk the array in key value pairs to
        # convert to a Ruby hash
        expect(redis.hgetall("x")).to eql({})
      end
    end

    context "when the db key already exists" do
      it "returns an array of keys and values" do
        redis.hmset("x", "y", "z", "1", "2")
        expect(redis.hgetall("x")).to eql("y" => "z", "1" => "2")
      end
    end
  end
end
