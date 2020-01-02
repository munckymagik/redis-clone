RSpec.describe "FLUSHDB", include_connection: true do
  it 'accepts one optional argument' do
    expect(redis.command("info", "flushdb")[0][1]).to eql(-1)
  end

  it "deletes all the keys of the currently selected DB" do
    redis.set("x", "123")
    redis.set("y", "456")

    redis.flushdb

    expect(redis.get("x")).to be_nil
    expect(redis.get("y")).to be_nil
  end
end
