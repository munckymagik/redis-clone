RSpec.describe "DEL", include_connection: true do
  it "removes the specified keys" do
    redis.set("x", "123")
    expect(redis.get("x")).to eql("123")

    redis.del("x")

    expect(redis.get("x")).to be_nil
  end
end
