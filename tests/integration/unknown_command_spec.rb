RSpec.describe "Unknown command", include_connection: true do
  context 'when the command has no arguments' do
    it "return an error string" do
      expect { redis.xyz }
        .to raise_error(
          "ERR unknown command `xyz`, with args beginning with:"
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
