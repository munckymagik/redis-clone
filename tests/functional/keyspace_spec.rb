RSpec.describe "Keyspace commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "del")[0][1]).to eql(-2)
      expect(redis.command("info", "exists")[0][1]).to eql(-2)
      expect(redis.command("info", "keys")[0][1]).to eql(2)
      expect(redis.command("info", "object")[0][1]).to eql(-2)
      expect(redis.command("info", "type")[0][1]).to eql(2)
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
      it "returns 'string' for string types" do
        redis.set("x", "123")
        expect(redis.type("x")).to eql("string")
        redis.incr("x")
        expect(redis.type("x")).to eql("string")
      end

      it "returns 'list' for list types" do
        redis.rpush("x", 1)
        expect(redis.type("x")).to eql("list")
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

          if using_real_redis?
            expect(redis.object("encoding", "a")).to eql("embstr")
            expect(redis.object("encoding", "b")).to eql("quicklist")
            expect(redis.object("encoding", "c")).to eql("int")
          else
            expect(redis.object("encoding", "a")).to eql("byte_string")
            expect(redis.object("encoding", "b")).to eql("vecdeque")
            expect(redis.object("encoding", "c")).to eql("int")
          end
        end
      end
    end
  end
end
