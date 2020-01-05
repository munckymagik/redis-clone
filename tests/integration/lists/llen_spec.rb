RSpec.describe "LLEN", include_connection: true do
  it "expects at exactly one argument" do
    expect(redis.command("info", "llen")[0][1]).to eql(2)

    expect { redis.call("llen") }.to raise_error(
      "ERR wrong number of arguments for 'llen' command"
    )
  end

  context "when the key does not already exist" do
    it "returns 0" do
      expect(redis.llen("x")).to eql(0)
    end
  end
end
