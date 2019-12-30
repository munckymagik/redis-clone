RSpec.describe "COMMAND", include_connection: true do
  describe "(no subcommand)" do
    it "returns the full list of supported commands" do
      output = redis.command
      expect(output.count).to eql(redis.command("count"))
      set_entry = output.find { |r| r.first == "set" }
      expect(set_entry[1]).to eql(-3)
    end
  end

  describe "HELP" do
    it "returns the help string" do
      output = redis.command("help")
      expect(output[0]).to match(/^\(no subcommand\)/)
      expect(output[1]).to match(/^COUNT/)
      expect(output[2]).to match(/^GETKEYS/)
      expect(output[3]).to match(/^INFO/)
    end
  end

  describe "COUNT" do
    it "returns the number of supported commands" do
      expect(redis.command("count")).to be_an(Integer)
        .and(be > 0)
    end
  end

  describe "GETKEYS" do
    it do
      pending "not implemented yet"
      fail
    end
  end

  describe "INFO" do
    it do
      pending "not implemented yet"
      fail
    end
  end

  context "when the subcommand is not supported" do
    it "returns a error" do
      expect { redis.command("xyz") }
        .to raise_error(
          /^ERR Unknown subcommand or wrong number of arguments for 'xyz'/
        )
    end
  end
end
