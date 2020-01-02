RSpec.describe "GET", include_connection: true do
  it 'expects at exactly one argument' do
    expect(redis.command("info", "get")[0][1]).to eql(2)

    expect { redis.call("get") }.to raise_error(
      "ERR wrong number of arguments for 'get' command"
    )
    expect { redis.call("get", "a", "b") }.to raise_error(
      "ERR wrong number of arguments for 'get' command"
    )
  end


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
