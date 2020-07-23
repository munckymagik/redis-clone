RSpec.describe "Keyspace commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "del").dig(0, 1)).to eql(-2)
      expect(redis.command("info", "exists").dig(0, 1)).to eql(-2)
      expect(redis.command("info", "expire").dig(0, 1)).to eql(3)
      expect(redis.command("info", "keys").dig(0, 1)).to eql(2)
      expect(redis.command("info", "object").dig(0, 1)).to eql(-2)
      expect(redis.command("info", "persist").dig(0, 1)).to eql(2)
      expect(redis.command("info", "ttl").dig(0, 1)).to eql(2)
      expect(redis.command("info", "type").dig(0, 1)).to eql(2)
    end
  end

  describe "KEYS" do
    context "when there are no keys" do
      it "returns an empty array" do
        expect(redis.keys('*')).to be_empty
      end
    end

    context "when there are keys" do
      let (:keynames) { %w[key_x key_y key_z foo_a foo_b foo_c].sort }

      before do
        keynames.each do |k|
          redis.set(k, "hello")
        end
      end

      it "returns an unsorted array of key names" do
        expect(redis.keys("*").sort).to eql(keynames)
      end

      context "when the argument is a glob pattern" do
        it "returns only the matching keys" do
          expect(redis.keys("foo_*").sort).to eql(%w[foo_a foo_b foo_c])
          expect(redis.keys("foo_[a-b]").sort).to eql(%w[foo_a foo_b])
          expect(redis.keys("foo_[ac]").sort).to eql(%w[foo_a foo_c])
          expect(redis.keys("*_x").sort).to eql(%w[key_x])
        end
      end

      context "when the glob pattern is invalid" do
        it "returns an empty result" do
          expect(redis.keys('[]')).to be_empty
          expect(redis.keys('[ab')).to be_empty
        end
      end

      context "when some of the keys have expired" do
        it "does not list the expired keys", slow: true do
          redis.expire("key_x", 1)
          sleep(1.5)
          expect(redis.keys("*").sort).to eql(keynames - %w[key_x])
        end
      end
    end
  end

  describe "DEL" do
    context "when the specified key exists" do
      it "removes the key and returns 1" do
        redis.set("x", "123")
        expect(redis.del("x")).to eql(1)
        expect(redis.exists("x")).to be(false)
      end
    end

    context "when the specified key does not exist" do
      it "removes the key and returns 0" do
        expect(redis.del("does-not-exist")).to eql(0)
        expect(redis.exists("does-not-exist")).to be(false)
      end
    end

    context "when there are multiple keys to remove" do
      it "removes the keys and returns the count of removed" do
        redis.set("x", "123")
        redis.set("y", "123")
        expect(redis.del("x", "y", "does-not-exist")).to eql(2)
        expect(redis.call("exists", "x", "y")).to eql(0)
      end
    end
  end

  describe "EXISTS" do
    it "checks the existence of the specified keys" do
      redis.set("a", "123")
      redis.set("b", "123")

      expect(redis.call("exists", "x")).to eql(0)
      expect(redis.call("exists", "a")).to eql(1)
      expect(redis.call("exists", "a", "b")).to eql(2)
      expect(redis.call("exists", "a", "a", "b")).to eql(3)
      expect(redis.call("exists", "a", "a", "x")).to eql(2)
    end
  end

  describe "TYPE" do
    context "when the specified key does not exist" do
      it "returns none" do
        expect(redis.type("does-not-exist")).to eql("none")
      end
    end

    context "when the specified key exists" do
      it "returns the type of value identified by the key" do
        redis.set("a", "abc")
        redis.set("b", "123")
        redis.set("c", 123)
        redis.rpush("d", 1)
        redis.hset("e", "y", 1)
        redis.sadd("f", "y")

        expect(redis.type("a")).to eql("string")
        expect(redis.type("b")).to eql("string")
        expect(redis.type("c")).to eql("string")
        expect(redis.type("d")).to eql("list")
        expect(redis.type("e")).to eql("hash")
        expect(redis.type("f")).to eql("set")
      end
    end
  end

  describe "OBJECT" do
    describe "HELP" do
      it "returns the help string" do
        output = redis.object("help")
        expect(output.count).to eql(5)
        expect(output[0]).to eql(
          "OBJECT <subcommand> arg arg ... arg. Subcommands are:"
        )
        expect(output[1]).to match(/^ENCODING/)
        expect(output[2]).to match(/^FREQ/)
        expect(output[3]).to match(/^IDLETIME/)
        expect(output[4]).to match(/^REFCOUNT/)
      end
    end

    describe "ENCODING" do
      context "when key is not specified" do
        it "replies with an error" do
          expect { redis.object("encoding") }
            .to raise_error(
              "ERR Unknown subcommand or wrong number of arguments for " \
              "'encoding'. Try OBJECT HELP."
            )
        end
      end

      context "when the specified key does not exist" do
        it "returns none" do
          expect(redis.object("encoding", "does-not-exist")).to be_nil
        end
      end

      context "when the specified key exists" do
        it "returns the encoding for values" do
          redis.set("a", "string")
          redis.rpush("b", %w[1 2])
          redis.set("c", 1)
          redis.hset("d", "x", "y")

          if using_real_redis?
            expect(redis.object("encoding", "a")).to eql("embstr")
            expect(redis.object("encoding", "b")).to eql("quicklist")
            expect(redis.object("encoding", "c")).to eql("int")
            expect(redis.object("encoding", "d")).to eql("ziplist")
          else
            expect(redis.object("encoding", "a")).to eql("byte_string")
            expect(redis.object("encoding", "b")).to eql("vecdeque")
            expect(redis.object("encoding", "c")).to eql("int")
            expect(redis.object("encoding", "d")).to eql("hash_map")
          end
        end
      end
    end
  end

  describe "EXPIRE" do
    context "when the specified key does not exist" do
      it "returns 0 (false)" do
        expect(redis.expire("does-not-exist", 10)).to be(false)
      end
    end

    context "when the specified key exists" do
      it "sets the expiration for the key" do
        redis.set("x", "abc")
        expect(redis.expire("x", 10)).to be(true)
      end
    end

    context "when the specified key exists and has a future expiry" do
      it "updates the expiration for the key" do
        redis.set("x", "abc")
        redis.expire("x", 10)
        redis.expire("x", 100)
        expect(redis.ttl("x")).to be_between(0, 100)
      end
    end

    context "when the specified key exists and ttl is not positive" do
      it "deletes the key" do
        redis.set("x", "abc")
        expect(redis.expire("x", 0)).to be(true)
        expect(redis.exists("x")).to be(false)

        redis.set("x", "abc")
        expect(redis.expire("x", -1)).to be(true)
        expect(redis.exists("x")).to be(false)
      end
    end

    context "when the specified key has already expired", slow: true do
      it "removes the key" do
        redis.set("x", "abc")
        expect(redis.expire("x", 1)).to be(true)
        sleep(1.5)
        expect(redis.expire("x", 1)).to be(false)
        expect(redis.exists("x")).to be(false)
      end
    end

    context "when ttl is not an integer" do
      it "replies with an error" do
        redis.set("x", "123")
        expect { redis.expire("x", "abc") }
          .to raise_error(
            "ERR value is not an integer or out of range"
          )
      end
    end
  end

  describe "PERSIST" do
    context "when the specified key does not exist" do
      it "returns 0 (false)" do
        expect(redis.persist("does-not-exist")).to be(false)
      end
    end

    context "when the specified key exists" do
      it "returns 1 (true)" do
        redis.set("x", "abc")
        redis.expire("x", 10)
        expect(redis.persist("x")).to be(true)
        expect(redis.ttl("x")).to eql(-1)
      end
    end

    context "when the specified key has already expired", slow: true do
      it "removes the key" do
        redis.set("x", "abc")
        expect(redis.expire("x", 1)).to be(true)
        sleep(1.5)
        expect(redis.persist("x")).to be(false)
        expect(redis.exists("x")).to be(false)
      end
    end
  end

  describe "TTL" do
    context "when the specified key does not exist" do
      it "returns -2" do
        expect(redis.ttl("does-not-exist")).to eql(-2)
      end
    end

    context "when the specified key has expired", slow: true do
      it "returns -2 and removes the key" do
        redis.set("x", "abc")
        redis.expire("x", 1)
        sleep(1.5)
        expect(redis.ttl("x")).to eql(-2)
        expect(redis.exists("x")).to be(false)
      end
    end

    context "when the specified key exists but has no associated expire" do
      it "returns -1" do
        redis.set("x", "abc")
        expect(redis.ttl("x")).to eql(-1)
      end
    end

    context "when the specified key exists and has an expire" do
      it "returns emaining time to live of the key" do
        redis.set("x", "abc")
        redis.expire("x", 10)
        expect(redis.ttl("x")).to be_between(0, 10)
      end
    end
  end
end
