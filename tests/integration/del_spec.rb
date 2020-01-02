RSpec.describe "DEL", include_connection: true do
  it 'expects at least one argument' do
    expect(redis.command("info", "del")[0][1]).to eql(-2)

    expect { redis.call("del") }.to raise_error(
      "ERR wrong number of arguments for 'del' command"
    )
  end

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
