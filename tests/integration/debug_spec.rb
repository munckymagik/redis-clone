RSpec.describe "DEBUG", include_connection: true do
  it 'expects at least one argument' do
    expect(redis.command("info", "debug")[0][1]).to eql(-2)

    expect { redis.call("debug") }.to raise_error(
      "ERR wrong number of arguments for 'debug' command"
    )
  end

  describe "HELP" do
    it "returns a help string" do
      result = redis.debug("help")
      expect(result.count).to be > 1
      expect(result[0]).to eql(
        "DEBUG <subcommand> arg arg ... arg. Subcommands are:"
      )
    end
  end

  describe "_CMD_ERROR" do
    it "reports a server error" do
      expect { redis.debug("_cmd_error") }
        .to raise_error("ERR server error")
    end
  end

  describe "PANIC" do
    it "reports a server error" do
      expect { redis.debug("panic") }
        .to raise_error("ERR server error")
    end
  end
end
