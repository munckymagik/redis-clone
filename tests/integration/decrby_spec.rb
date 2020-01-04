RSpec.describe "DECRBY", include_connection: true do
  it "expects 2 arguments" do
    expect(redis.command("info", "decrby")[0][1]).to eql(3)

    expect { redis.call("decrby") }.to raise_error(
      "ERR wrong number of arguments for 'decrby' command"
    )
    expect { redis.call("decrby", "x") }.to raise_error(
      "ERR wrong number of arguments for 'decrby' command"
    )
  end

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
