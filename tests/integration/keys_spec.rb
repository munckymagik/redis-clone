RSpec.describe "KEYS", include_connection: true do
  it 'expects exactly one argument' do
    expect(redis.command("info", "keys")[0][1]).to eql(2)

    expect { redis.call("keys") }.to raise_error(
      "ERR wrong number of arguments for 'keys' command"
    )
  end

  context "when there are no keys" do
    it "returns an empty array" do
      expect(redis.keys('*')).to be_empty
    end
  end

  context "when there are keys" do
    let (:keynames) { %w[key_x key_y key_z foo_a foo_b foo_c].sort }

    before do
      keynames.each do |k|
        redis.set(k, "hello")
      end
    end

    it "returns an unsorted array of key names" do
      expect(redis.keys("*").sort).to eql(keynames)
    end

    context "when the argument is a glob pattern" do
      it "returns only the matching keys" do
        expect(redis.keys("foo_*").sort).to eql(%w[foo_a foo_b foo_c])
        expect(redis.keys("foo_[a-b]").sort).to eql(%w[foo_a foo_b])
        expect(redis.keys("foo_[ac]").sort).to eql(%w[foo_a foo_c])
        expect(redis.keys("*_x").sort).to eql(%w[key_x])
      end
    end

    context "when the glob pattern is invalid" do
      it "returns an empty result" do
        expect(redis.keys('[]')).to be_empty
        expect(redis.keys('[ab')).to be_empty
      end
    end
  end
end
