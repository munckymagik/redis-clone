RSpec.describe "DEL", include_connection: true do
  it 'expects at least one argument' do
    expect(redis.command("info", "del")[0][1]).to eql(-2)

    expect { redis.call("del") }.to raise_error(
      "ERR wrong number of arguments for 'del' command"
    )
  end

  it "removes the specified keys" do
    redis.set("x", "123")
    expect(redis.get("x")).to eql("123")

    redis.del("x")

    expect(redis.get("x")).to be_nil
  end
end
