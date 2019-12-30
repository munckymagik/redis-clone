RSpec.describe "SET", include_connection: true do
  it "sets a key and value" do
    expect(redis.get("x")).to be_nil
    expect(redis.set("x", "123")).to eql("OK")
    expect(redis.get("x")).to eql("123")
  end
end
