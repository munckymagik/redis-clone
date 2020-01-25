RSpec.describe "Server commands", include_connection: true do
  describe "COMMAND" do
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
        expect(output.count).to eql(4)
        expect(output[0]).to eql(
          "COMMAND <subcommand> arg arg ... arg. Subcommands are:"
        )
        expect(output[1]).to match(/^\(no subcommand\)/)
        expect(output[2]).to match(/^COUNT/)
        expect(output[3]).to match(/^INFO/)
      end
    end

    describe "COUNT" do
      it "returns the number of supported commands" do
        expect(redis.command("count")).to be_an(Integer)
          .and(be > 0)
      end
    end

    describe "INFO" do
      it "returns the requested subset of supported commands" do
        requested = %w[set get]
        output = redis.command("info", "set", "get")
        expect(output.count).to eql(2)
        expect(output[0].first).to eql("set")
        expect(output[1].first).to eql("get")
      end

      context "when given no arguments" do
        it "returns an empty array" do
          output = redis.command("info")
          expect(output).to be_empty
        end
      end

      context "when a non-existent command is requested" do
        it "returns an null array for the unrecognised command" do
          output = redis.command("info", "xxx", "set")
          expect(output.count).to eql(2)
          expect(output[0]).to be_nil
          expect(output[1].first).to eql("set")
        end
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

  describe "DEBUG" do
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
  end

  describe "FLUSHDB" do
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
end
