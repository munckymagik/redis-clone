RSpec.describe "SET", include_connection: true do
  it 'expects at least 2 arguments' do
    expect(redis.command("info", "set")[0][1]).to eql(-3)

    expect { redis.call("set") }.to raise_error(
      "ERR wrong number of arguments for 'set' command"
    )
    expect { redis.call("set", "x") }.to raise_error(
      "ERR wrong number of arguments for 'set' command"
    )
    expect { redis.call("set", "x", "y") }.not_to raise_error
  end

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
