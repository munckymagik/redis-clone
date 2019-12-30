RSpec.describe "GET", include_connection: true do
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
