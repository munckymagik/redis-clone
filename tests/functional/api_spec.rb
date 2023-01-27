RSpec.describe "General API behaviour", include_connection: true do
  describe "Arity validation" do
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

  describe "Unknown commands" do
    context 'when the command has no arguments' do
      it "return an error string" do
        expect { redis.xyz }
          .to raise_error(
            "ERR unknown command `xyz`, with args beginning with: "
          )
      end
    end

    context 'when the command has arguments' do
      it "return an error string" do
        expect { redis.xyz(1, 2, 3) }
          .to raise_error(
            "ERR unknown command `xyz`, with args beginning with: `1`, `2`, `3`,"
          )
      end
    end
  end

  describe "Error handling", redis_clone_only: true do
    specify "when there is a command error it reports a server error" do
      expect { redis.debug("error") }
        .to raise_error("ERR server error")
    end

    specify "when there is a command panic it reports a server error" do
      expect { redis.debug("panic") }
        .to raise_error("ERR server error")
    end
  end
end
