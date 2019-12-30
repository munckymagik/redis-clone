RSpec.describe "SET", include_connection: true do
  it "sets a key and value" do
    reply = redis.set("x", "123")
    expect(reply).to eql("OK")

    value = redis.get("x")
    expect(value).to eql("123")
  end
end
