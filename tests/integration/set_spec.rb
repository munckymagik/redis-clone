RSpec.describe "SET", include_connection: true do
  context "when the key does not exist" do
    it "sets the key to hold the string value" do
      expect(redis.get("x")).to be_nil
      expect(redis.set("x", "123")).to eql("OK")
      expect(redis.get("x")).to eql("123")
    end
  end

  context "when key already holds a value" do
    it "it is overwritten" do
      redis.set("x", "123")
      expect(redis.get("x")).to eql("123")

      expect(redis.set("x", "456")).to eql("OK")
      expect(redis.get("x")).to eql("456")
    end
  end
end