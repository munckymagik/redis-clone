RSpec.describe "FLUSHDB", include_connection: true do
  it "deletes all the keys of the currently selected DB" do
    redis.set("x", "123")
    redis.set("y", "456")

    redis.flushdb

    expect(redis.get("x")).to be_nil
    expect(redis.get("y")).to be_nil
  end
end
