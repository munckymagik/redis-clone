RSpec.describe "LPUSH", include_connection: true do
  it "expects at least two arguments" do
    expect(redis.command("info", "lpush")[0][1]).to eql(-3)

    expect { redis.call("lpush") }.to raise_error(
      "ERR wrong number of arguments for 'lpush' command"
    )
    expect { redis.call("lpush", "x") }.to raise_error(
      "ERR wrong number of arguments for 'lpush' command"
    )
  end

  context "when the key does not already exist" do
    it "creates a new list key and adds the new element" do
      expect(redis.lpush("x", "abc")).to eql(1)
      expect(redis.exists("x")).to be(true)
    end
  end
end
