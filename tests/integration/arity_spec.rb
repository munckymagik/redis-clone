RSpec.describe "arity validation", include_connection: true do
  describe "commands with exact arity" do
    it "ensures the exact amount of arguments are present" do
      # get requires exactly 1 argument
      expect { redis.call("get") }.to raise_error(
        "ERR wrong number of arguments for 'get' command"
      )

      expect(redis.get("x")).to be_nil
    end
  end

  describe "commands with minimum arity" do
    it "ensures the exact amount of arguments are present" do
      # set requires at least 2 arguments
      expect { redis.call("set", "x") }.to raise_error(
        "ERR wrong number of arguments for 'set' command"
      )

      expect(redis.set("x", "1")).to eql("OK")
    end
  end
end
