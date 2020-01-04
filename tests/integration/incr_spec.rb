RSpec.describe "INCR", include_connection: true do
  it "expects one argument" do
    expect(redis.command("info", "incr")[0][1]).to eql(2)

    expect { redis.call("incr") }.to raise_error(
      "ERR wrong number of arguments for 'incr' command"
    )
  end

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
