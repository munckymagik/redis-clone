RSpec.describe "INCRBY", include_connection: true do
  it "expects 2 arguments" do
    expect(redis.command("info", "incrby")[0][1]).to eql(3)

    expect { redis.call("incrby") }.to raise_error(
      "ERR wrong number of arguments for 'incrby' command"
    )
    expect { redis.call("incrby", "x") }.to raise_error(
      "ERR wrong number of arguments for 'incrby' command"
    )
  end

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
