RSpec.describe "Strings commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "set").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "get").dig(0, 1)).to eql(2)
      expect(redis.command("info", "mset").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "mget").dig(0, 1)).to eql(-2)
      expect(redis.command("info", "incr").dig(0, 1)).to eql(2)
      expect(redis.command("info", "incrby").dig(0, 1)).to eql(3)
      expect(redis.command("info", "decr").dig(0, 1)).to eql(2)
      expect(redis.command("info", "decrby").dig(0, 1)).to eql(3)
    end
  end

  describe "commands used against the wrong type" do
    let(:expected_error) { "WRONGTYPE Operation against a key holding the wrong kind of value" }

    specify "all but SET raise an error" do
      redis.rpush("x", 1)

      expect { redis.get("x") }
        .to raise_error(expected_error)
      expect { redis.incr("x") }
        .to raise_error(expected_error)
      expect { redis.incrby("x", 1) }
        .to raise_error(expected_error)
      expect { redis.decr("x") }
        .to raise_error(expected_error)
      expect { redis.decrby("x", 1) }
        .to raise_error(expected_error)
    end

    specify "SET overwrites with the new type" do
      redis.rpush("x", 1)

      expect(redis.set("x", "y")).to eql("OK")
      expect(redis.get("x")).to eql("y")
    end
  end

  describe "SET" do
    context "when the key does not exist" do
      it "sets the key to hold the string value" do
        expect(redis.set("x", "abc")).to eql("OK")
        expect(redis.get("x")).to eql("abc")
      end

      it "sets the key to hold the string value if NX is specified" do
        expect(redis.set("x", "123", nx: true)).to be(true)
        expect(redis.get("x")).to eql("123")
      end

      it "does not set the key if XX is specified" do
        expect(redis.set("x", "abc", xx: true)).to be(false)
        expect(redis.exists?("x")).to be(false)
      end
    end

    context "when key already holds a value" do
      before do
        redis.set("x", "123")
      end

      it "is overwritten" do
        expect(redis.set("x", "456")).to eql("OK")
        expect(redis.get("x")).to eql("456")
      end

      it "is overwritten if XX is specified" do
        expect(redis.set("x", "456", xx: true)).to be(true)
        expect(redis.get("x")).to eql("456")
      end

      it "is not overwritten if NX is specified" do
        expect(redis.set("x", "456", nx: true)).to be(false)
        expect(redis.get("x")).to eql("123")
      end
    end

    context "if both nx and xx are specified" do
      it "responds with a syntax error" do
        expect { redis.set("x", "y", xx: true, nx: true) }
          .to raise_error("ERR syntax error")
      end
    end

    describe "setting expire times" do
      context "when the key does not exist" do
        it "sets a ttl if EX is specified" do
          redis.set("x", "a", ex: 10)
          expect(redis.ttl("x")).to be_between(0, 10)
        end

        it "sets a ttl if PX is specified" do
          redis.set("x", "a", px: 20_000)
          expect(redis.ttl("x")).to be_between(10, 20)
        end
      end

      context "when the key already exists but is persistant" do
        it "sets a ttl if EX is specified" do
          redis.set("x", "a")
          expect(redis.ttl("x")).to eql(-1)
          redis.set("x", "a", ex: 10)
          expect(redis.ttl("x")).to be_between(0, 10)
        end

        it "sets a ttl if PX is specified" do
          redis.set("x", "a")
          expect(redis.ttl("x")).to eql(-1)
          redis.set("x", "a", px: 20_000)
          expect(redis.ttl("x")).to be_between(10, 20)
        end
      end

      context "when the key already exists and has a ttl set" do
        it "overwrites the ttl if EX is specified" do
          redis.set("x", "a", ex: 10)
          expect(redis.ttl("x")).to be_between(0, 10)
          redis.set("x", "a", ex: 20)
          expect(redis.ttl("x")).to be_between(10, 20)
        end

        it "overwrites the ttl if PX is specified" do
          redis.set("x", "a", px: 10_000)
          expect(redis.ttl("x")).to be_between(0, 10)
          redis.set("x", "a", px: 20_000)
          expect(redis.ttl("x")).to be_between(10, 20)
        end
      end

      context "when the tll is not positive" do
        it "returns an error" do
          expect {
            redis.set("x", "a", ex: 0)
          }.to raise_error("ERR invalid expire time in set")

          expect {
            redis.set("x", "a", ex: -1)
          }.to raise_error("ERR invalid expire time in set")

          expect {
            redis.set("x", "a", px: 0)
          }.to raise_error("ERR invalid expire time in set")

          expect {
            redis.set("x", "a", px: -1)
          }.to raise_error("ERR invalid expire time in set")
        end
      end

      context "when the ttl is missing" do
        it "returns an error" do
          expect {
            redis.call("set", "x", "a", "ex")
          }.to raise_error("ERR syntax error")

          expect {
            redis.call("set", "x", "a", "px")
          }.to raise_error("ERR syntax error")
        end
      end

      context "if both ex and px are specified" do
        it "responds with a syntax error" do
          expect { redis.set("x", "y", ex: 1, px: 1000) }
            .to raise_error("ERR syntax error")
        end
      end
    end

    describe "combinations of arguments" do
      it "works" do
        # Can use ex and nx together
        redis.call("set", "key", "val", "ex", 10, "nx")
        expect(redis.ttl("key")).to be_between(0, 10)

        # Can use ex and xx together
        redis.call("set", "key", "val", "ex", 10, "xx")
        expect(redis.ttl("key")).to be_between(0, 10)

        # For an existing key, nx prevents the value changing but not the ttl
        redis.call("set", "key", "xyz", "px", 20_000, "nx")
        expect(redis.get("key")).to eql("val") # value does not change
        expect(redis.ttl("key")).to be_between(10, 20)

        # For an existing key with xx, both the value and the ttl change
        redis.call("set", "key", "val", "px", 30_000, "xx")
        expect(redis.ttl("key")).to be_between(20, 30)

        # The order of the arguments does not matter
        redis.call("set", "key", "xyz", "nx", "px", 40_000)
        expect(redis.get("key")).to eql("val") # value does not change
        expect(redis.ttl("key")).to be_between(30, 40)
      end
    end

    it "encodes numeric strings as integers" do
      redis.set("x", "a")
      expect(redis.object("encoding", "x")).to eql(
        using_real_redis? ? "embstr" : "byte_string"
      )
      redis.del("x")

      redis.set("x", "-1")
      expect(redis.object("encoding", "x")).to eql("int")
      redis.set("x", "0")
      expect(redis.object("encoding", "x")).to eql("int")
      redis.set("x", "1")
      expect(redis.object("encoding", "x")).to eql("int")
    end

    it "supports binary data in the key and the value" do
      # Invalid UTF-8 sequence sourced from:
      #   https://stackoverflow.com/a/3886015/369171
      expect(redis.set("\xe2\x28\xa1", "\xe2\x28\xa1")).to eql("OK")
      expected = "\xe2\x28\xa1"
      result = redis.get("\xe2\x28\xa1")
      expect(result.unpack('C*')).to eql(expected.unpack('C*'))
    end

    it "permits zero length keys and values" do
      expect(redis.set("", "")).to eql("OK")
      expect(redis.get("")).to eql("")
    end
  end

  describe "GET" do
    context 'when the specific key does not exist' do
      it "return nil" do
        expect(redis.get("non-existent")).to be_nil
      end
    end

    context "when the specific key exists" do
      it "returns its value" do
        redis.set("x", "123")
        expect(redis.get("x")).to eql("123")
      end
    end
  end

  describe "MSET" do
    context "when the key does not already exist" do
      it "creates one" do
        expect(redis.mset("x", "y")).to eql("OK")
        expect(redis.type("x")).to eql("string")
      end
    end

    context "when the key already exists" do
      it "overwrites the value" do
        redis.set("x", "y")
        expect(redis.get("x")).to eql("y")
        redis.mset("x", "yy")
        expect(redis.get("x")).to eql("yy")
      end
    end

    describe "variable arguments" do
      context "when only new keys are being added" do
        it "adds the keys" do
          expect(redis.mset("a", 1, "b", 2)).to eql("OK")

          expect(redis.get("a")).to eql("1")
          expect(redis.get("b")).to eql("2")
        end
      end

      context "when only some new keys are being added and others being updated" do
        it "adds the new keys and updates the existing" do
          redis.set("a", 1)
          expect(redis.mset("a", 2, "b", 2)).to eql("OK")

          expect(redis.get("a")).to eql("2")
          expect(redis.get("b")).to eql("2")
        end
      end

      context "when there is an uneven number of arguments" do
        it "returns an error" do
          expect {
              expect(redis.mset("a", 1, "b")).to eq("OK")
          }.to raise_error("ERR wrong number of arguments for MSET")
        end
      end
    end
  end

  describe "MGET" do
    context "when none of the keys exist" do
      it "returns an array of nils" do
        expect(redis.mget("x", "y")).to eql([nil, nil])
      end
    end

    context "when the some of the keys already exist" do
      it "returns values where they are found or nil where they don't exist" do
        redis.mset("a", "one", "c", "3")
        expect(redis.mget("a", "b", "c")).to eql(["one", nil, "3"])
      end
    end

    context "when any of the keys contains a non-string type value" do
      it "returns values where they are strings or nil where they are not" do
        redis.mset("a", "one", "c", "3")
        redis.rpush("b", %w[1])
        expect(redis.mget("a", "b", "c")).to eql(["one", nil, "3"])
      end
    end
  end

  describe "INCR" do
    context "when the key does not already exist" do
      it "creates the key and sets it to 1" do
        expect(redis.incr("x")).to eql(1)
        expect(redis.get("x")).to eql("1")
      end
    end

    context "when the key does already exists" do
      context "when the key was created by INCR" do
        it "increments the value" do
          expect(redis.incr("x")).to eql(1)
          expect(redis.incr("x")).to eql(2)
        end
      end

      context "when the key was created by SET" do
        it "increments the value" do
          redis.set("x", "100")
          expect(redis.incr("x")).to eql(101)
        end
      end
    end

    context "if value is INCR beyond i64 max" do
      it "returns an error" do
        redis.set("x", "9223372036854775807")
        expect { redis.incr("x") }
          .to raise_error("ERR increment or decrement would overflow")
        expect(redis.get("x")).to eql("9223372036854775807")
      end
    end

    context "when when there are leading spaces" do
      it "returns an error" do
        redis.set("x", " 1")
        expect { redis.incr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end

    context "when when there are trailing spaces" do
      it "returns an error" do
        redis.set("x", "1 ")
        expect { redis.incr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end

    context "when when there are both trailing and leading spaces" do
      it "returns an error" do
        redis.set("x", " 1 ")
        expect { redis.incr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end
  end

  describe "INCRBY" do
    context "when the key does not already exist" do
      it "creates the key and sets it to the requested increment" do
        expect(redis.incrby("x", 2)).to eql(2)
        expect(redis.get("x")).to eql("2")
      end
    end

    context "when the key does already exist" do
      it "increments the value by the requested increment" do
        expect(redis.set("x", 100))
        expect(redis.incrby("x", 2)).to eql(102)
      end
    end

    context "when the increment is non-numeric" do
      it "returns an error" do
        expect(redis.set("x", "nnn"))
        expect { redis.incrby("x", 2) }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end
  end

  describe "DECR" do
    context "when the key does not already exist" do
      it "creates the key and sets it to 1" do
        expect(redis.decr("x")).to eql(-1)
        expect(redis.get("x")).to eql("-1")
      end
    end

    context "when the key does already exists" do
      context "when the key was created by INCR" do
        it "decrements the value" do
          expect(redis.decr("x")).to eql(-1)
          expect(redis.decr("x")).to eql(-2)
        end
      end

      context "when the key was created by SET" do
        it "decrements the value" do
          redis.set("x", "100")
          expect(redis.decr("x")).to eql(99)
        end
      end
    end

    context "if value is DECR below i64 min" do
      it "replies with an error" do
        redis.set("x", "-9223372036854775808")
        expect { redis.decr("x") }
          .to raise_error("ERR increment or decrement would overflow")
        expect(redis.get("x")).to eql("-9223372036854775808")
      end
    end

    context "when when there are leading spaces" do
      it "returns an error" do
        redis.set("x", " 1")
        expect { redis.decr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end

    context "when when there are trailing spaces" do
      it "returns an error" do
        redis.set("x", "1 ")
        expect { redis.decr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end

    context "when when there are both trailing and leading spaces" do
      it "returns an error" do
        redis.set("x", " 1 ")
        expect { redis.decr("x") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end
  end

  describe "DECRBY" do
    context "when the key does not already exist" do
      it "creates the key and sets it to the requested decrement" do
        expect(redis.decrby("x", 2)).to eql(-2)
        expect(redis.get("x")).to eql("-2")
      end
    end

    context "when the key does already exist" do
      it "decrements the value by the requested decrement" do
        expect(redis.set("x", 100))
        expect(redis.decrby("x", 2)).to eql(98)
      end
    end

    context "when the decrement is non-numeric" do
      it "returns an error" do
        expect(redis.set("x", "nnn"))
        expect { redis.decrby("x", 2) }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end
  end
end
