RSpec.describe "EXISTS", include_connection: true do
  it 'expects at least one argument' do
    expect(redis.command("info", "exists")[0][1]).to eql(-2)

    expect { redis.call("exists") }.to raise_error(
      "ERR wrong number of arguments for 'exists' command"
    )
  end

  it "checks the existence of the specified keys" do
    redis.set("a", "123")
    redis.set("b", "123")

    expect(redis.call("exists", "x")).to eql(0)
    expect(redis.call("exists", "a")).to eql(1)
    expect(redis.call("exists", "a", "b")).to eql(2)
    expect(redis.call("exists", "a", "a", "b")).to eql(3)
    expect(redis.call("exists", "a", "a", "x")).to eql(2)
  end
end
