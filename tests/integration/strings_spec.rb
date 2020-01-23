RSpec.describe "Strings commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "set")[0][1]).to eql(-3)
      expect(redis.command("info", "get")[0][1]).to eql(2)
      expect(redis.command("info", "incr")[0][1]).to eql(2)
      expect(redis.command("info", "incrby")[0][1]).to eql(3)
      expect(redis.command("info", "decr")[0][1]).to eql(2)
      expect(redis.command("info", "decrby")[0][1]).to eql(3)
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
        expect(redis.set("x", "123")).to eql("OK")
        expect(redis.get("x")).to eql("123")
      end

      it "sets the key to hold the string value if NX is specified" do
        expect(redis.set("x", "123", nx: true)).to be(true)
        expect(redis.get("x")).to eql("123")
      end

      it "does not set the key if XX is specified" do
        expect(redis.set("x", "123", xx: true)).to be(false)
        expect(redis.exists("x")).to be(false)
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
      it 'responds with a syntax error' do
        expect { redis.set("x", "y", xx: true, nx: true) }
          .to raise_error("ERR syntax error")
      end
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
