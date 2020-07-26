RSpec.describe "List commands", include_connection: true do
  describe "arity" do
    specify "the arity for each command is correctly specified" do
      expect(redis.command("info", "rpush").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "lpush").dig(0, 1)).to eql(-3)
      expect(redis.command("info", "linsert").dig(0, 1)).to eql(5)
      expect(redis.command("info", "rpop").dig(0, 1)).to eql(2)
      expect(redis.command("info", "lpop").dig(0, 1)).to eql(2)
      expect(redis.command("info", "llen").dig(0, 1)).to eql(2)
      expect(redis.command("info", "lindex").dig(0, 1)).to eql(3)
      expect(redis.command("info", "lset").dig(0, 1)).to eql(4)
      expect(redis.command("info", "lrange").dig(0, 1)).to eql(4)
      expect(redis.command("info", "ltrim").dig(0, 1)).to eql(4)
      expect(redis.command("info", "lrem").dig(0, 1)).to eql(4)
    end
  end

  describe "commands used against the wrong type" do
    let(:expected_error) { "WRONGTYPE Operation against a key holding the wrong kind of value" }

    specify "raise an error" do
      redis.set("x", "not a list")

      expect { redis.rpush("x", "y") }
        .to raise_error(expected_error)
      expect { redis.lpush("x", "y") }
        .to raise_error(expected_error)
      expect { redis.linsert("x", "BEFORE", "1", "0") }
        .to raise_error(expected_error)
      expect { redis.rpop("x") }
        .to raise_error(expected_error)
      expect { redis.lpop("x") }
        .to raise_error(expected_error)
      expect { redis.llen("x") }
        .to raise_error(expected_error)
      expect { redis.lindex("x", 0) }
        .to raise_error(expected_error)
      expect { redis.lset("x", 0, "y") }
        .to raise_error(expected_error)
      expect { redis.lrange("x", 0, -1) }
        .to raise_error(expected_error)
      expect { redis.ltrim("x", 0, -1) }
        .to raise_error(expected_error)
      expect { redis.lrem("x", 1, "y") }
        .to raise_error(expected_error)
      expect(redis.exists("x")).to be(true)
    end
  end

  describe "LPUSH, RPUSH, LLEN, LINDEX, LPOP" do
    # Taken and converted from tests/unit/type/list.tcl in the Redis source
    specify "they work together as expected" do
      # first lpush then rpush
      expect(redis.lpush("myziplist1", "aa")).to eql(1)
      expect(redis.rpush("myziplist1", "bb")).to eql(2)
      expect(redis.rpush("myziplist1", "cc")).to eql(3)
      expect(redis.llen("myziplist1")).to eql(3)
      expect(redis.lindex("myziplist1", 0)).to eql("aa")
      expect(redis.lindex("myziplist1", 1)).to eql("bb")
      expect(redis.lindex("myziplist1", 2)).to eql("cc")
      expect(redis.lindex("myziplist1", 3)).to eql(nil)
      expect(redis.rpop("myziplist1")).to eql("cc")
      expect(redis.lpop("myziplist1")).to eql("aa")

      # first rpush then lpush
      expect(redis.rpush("myziplist2", "a")).to eql(1)
      expect(redis.lpush("myziplist2", "b")).to eql(2)
      expect(redis.lpush("myziplist2", "c")).to eql(3)
      expect(redis.llen("myziplist2")).to eql(3)
      expect(redis.lindex("myziplist2", 0)).to eql("c")
      expect(redis.lindex("myziplist2", 1)).to eql("b")
      expect(redis.lindex("myziplist2", 2)).to eql("a")
      expect(redis.lindex("myziplist2", 3)).to eql(nil)
      expect(redis.rpop("myziplist2")).to eql("a")
      expect(redis.lpop("myziplist2")).to eql("c")
    end
  end

  describe "LPUSH and RPUSH" do
    context "when the key does not already exist" do
      specify "LPUSH creates a new key as a list and adds the new element" do
        expect(redis.lpush("x", "abc")).to eql(1)
        expect(redis.exists("x")).to be(true)
      end

      specify "RPUSH creates a new key as a list and adds the new element" do
        expect(redis.rpush("x", "abc")).to eql(1)
        expect(redis.exists("x")).to be(true)
      end
    end

    context "when the key already exists" do
      specify "LPUSH adds an element to the left hand side of the list" do
        expect(redis.rpush("x", "1")).to eql(1)
        expect(redis.lpush("x", "2")).to eql(2)
        expect(redis.lrange("x", 0, -1)).to eql(%w[2 1])
      end

      specify "RPUSH adds an element to the right hand side of the list" do
        expect(redis.lpush("x", "1")).to eql(1)
        expect(redis.rpush("x", "2")).to eql(2)
        expect(redis.lrange("x", 0, -1)).to eql(%w[1 2])
      end
    end

    describe "variable arguments" do
      specify "LPUSH pushes 1 or more elements on the left" do
        expect(redis.lpush("x", %w[5 4])).to eql(2)
        expect(redis.lpush("x", %w[3 2 1])).to eql(5)
        expect(redis.lrange("x", 0, -1)).to eql(%w[1 2 3 4 5])
      end

      specify "RPUSH pushes 1 or more elements on the right" do
        expect(redis.rpush("x", %w[1 2])).to eql(2)
        expect(redis.rpush("x", %w[3 4 5])).to eql(5)
        expect(redis.lrange("x", 0, -1)).to eql(%w[1 2 3 4 5])
      end
    end
  end

  describe "LLEN" do
    context "when the key does not already exist" do
      it "returns 0" do
        expect(redis.llen("x")).to eql(0)
      end
    end

    context "when the key does exist" do
      it "returns the size" do
        redis.rpush("x", [1, 2, 3])
        expect(redis.llen("x")).to eql(3)
        redis.rpop("x")
        expect(redis.llen("x")).to eql(2)
      end
    end
  end

  describe "LRANGE" do
    context "when the key does not already exist" do
      it "returns an empty array" do
        expect(redis.lrange("x", 0, -1)).to be_empty
      end
    end

    context "when the key exists" do
      before do
        redis.rpush("x", [1, 2, 3])
      end

      specify "ranging over the start index" do
        expect(redis.lrange("x", -4, 2)).to eql(%w[1 2 3])
        expect(redis.lrange("x", -3, 2)).to eql(%w[1 2 3])
        expect(redis.lrange("x", -2, 2)).to eql(%w[2 3])
        expect(redis.lrange("x", -1, 2)).to eql(%w[3])
        expect(redis.lrange("x",  0, 2)).to eql(%w[1 2 3])
        expect(redis.lrange("x",  1, 2)).to eql(%w[2 3])
        expect(redis.lrange("x",  2, 2)).to eql(%w[3])
        expect(redis.lrange("x",  3, 2)).to eql(%w[])
      end

      specify "ranging over the end index" do
        expect(redis.lrange("x", 0, -4)).to eql(%w[])
        expect(redis.lrange("x", 0, -3)).to eql(%w[1])
        expect(redis.lrange("x", 0, -2)).to eql(%w[1 2])
        expect(redis.lrange("x", 0, -1)).to eql(%w[1 2 3])
        expect(redis.lrange("x", 0, 0)).to eql(%w[1])
        expect(redis.lrange("x", 0, 1)).to eql(%w[1 2])
        expect(redis.lrange("x", 0, 2)).to eql(%w[1 2 3])
        expect(redis.lrange("x", 0, 3)).to eql(%w[1 2 3])
      end

      specify "ranging over both in lock step" do
        expect(redis.lrange("x", -4, -4)).to eql(%w[])
        expect(redis.lrange("x", -3, -3)).to eql(%w[1])
        expect(redis.lrange("x", -2, -2)).to eql(%w[2])
        expect(redis.lrange("x", -1, -1)).to eql(%w[3])
        expect(redis.lrange("x", 0, 0)).to eql(%w[1])
        expect(redis.lrange("x", 1, 1)).to eql(%w[2])
        expect(redis.lrange("x", 2, 2)).to eql(%w[3])
        expect(redis.lrange("x", 3, 3)).to eql(%w[])
      end

      specify "when start > end or start > len" do
        expect(redis.lrange("x", 2, 1)).to eql(%w[])
        expect(redis.lrange("x", 4, 4)).to eql(%w[])
        expect(redis.lrange("x", 4, 5)).to eql(%w[])
      end
    end
  end

  describe "LINSERT" do
    context "when the key does not already exist" do
      it "is a no-op and returns 0" do
        expect(redis.linsert("x", "BEFORE", "1", "0")).to eql(0)
        expect(redis.exists("x")).to be(false)
      end
    end

    context "when the key does exist" do
      it "inserts the elements returns the size" do
        redis.rpush("x", %w[1 4])
        expect(redis.linsert("x", "BEFORE", "1", "0")).to eql(3)
        expect(redis.lrange("x", 0, -1)).to eql(%w[0 1 4])
        expect(redis.linsert("x", "AFTER", "1", "2")).to eql(4)
        expect(redis.lrange("x", 0, -1)).to eql(%w[0 1 2 4])
        expect(redis.linsert("x", "BEFore", "4", "3")).to eql(5)
        expect(redis.lrange("x", 0, -1)).to eql(%w[0 1 2 3 4])
        expect(redis.linsert("x", "AFter", "4", "5")).to eql(6)
        expect(redis.lrange("x", 0, -1)).to eql(%w[0 1 2 3 4 5])
      end
    end

    context "when arg 2 is not BEFORE or AFTER" do
      it "returns an error" do
        redis.rpush("x", %w[1])
        expect { redis.linsert("x", "bang", "1", "0") }
          .to raise_error("ERR syntax error")
      end
    end

    context "when pivot is not found" do
      it "returns -1 and does not modify the list" do
        redis.rpush("x", %w[1])
        expect(redis.linsert("x", "BEFORE", "7", "0")).to eql(-1)
        expect(redis.lrange("x", 0, -1)).to eql(%w[1])
      end
    end
  end

  describe "LPOP and RPOP" do
    context "when the key does not exist" do
      specify "they return nil" do
        expect(redis.lpop("x")).to be_nil
        expect(redis.rpop("x")).to be_nil
      end
    end

    context "when the key exists" do
      specify "LPOP removes and returns the leftmost element" do
        redis.rpush("x", %w[1 2])

        expect(redis.lpop("x")).to eql("1")
        expect(redis.lrange("x", 0, -1)).to eql(%w[2])
      end

      specify "RPOP removes and returns the rightmost element" do
        redis.rpush("x", %w[1 2])

        expect(redis.rpop("x")).to eql("2")
        expect(redis.lrange("x", 0, -1)).to eql(%w[1])
      end
    end
  end

  describe "LINDEX" do
    context "when the key does not exist" do
      it "returns nil" do
        expect(redis.lindex("x", 0)).to be_nil
      end
    end

    context "when the key exists" do
      it "returns the value at the index or nil if out of range" do
        redis.rpush("x", %w[1 2])

        expect(redis.lindex("x", 0)).to eql("1")
        expect(redis.lindex("x", 1)).to eql("2")
        expect(redis.lindex("x", 2)).to be_nil
      end
    end

    context "when the index is negative" do
      it "returns the value relative to the end or nil if out of range" do
        redis.rpush("x", %w[1 2])

        expect(redis.lindex("x", -1)).to eql("2")
        expect(redis.lindex("x", -2)).to eql("1")
        expect(redis.lindex("x", -3)).to be_nil
      end
    end

    context "when index is not a number" do
      it "returns an error" do
        redis.rpush("x", %w[1])

        expect { redis.lindex("x", "bang") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end
  end

  describe "LSET" do
    context "when the key does not exist" do
      it "returns an error" do
        expect { redis.lset("x", 0, 1) }
          .to raise_error("ERR no such key")
      end
    end

    context "when the key exists" do
      it "sets the values at index to the new value given" do
        redis.rpush("x", %w[1 2])

        expect(redis.lset("x", 0, "one")).to eql("OK")
        expect(redis.lset("x", 1, "two")).to eql("OK")
        expect(redis.lrange("x", 0, -1)).to eql(%w[one two])
      end
    end

    context "when the index is negative" do
      it "sets the values at index to the new value given" do
        redis.rpush("x", %w[1 2])

        expect(redis.lset("x", -1, "two")).to eql("OK")
        expect(redis.lset("x", -2, "one")).to eql("OK")
        expect(redis.lrange("x", 0, -1)).to eql(%w[one two])
      end
    end

    context "when index is not a number" do
      it "returns an error" do
        redis.rpush("x", %w[1])

        expect { redis.lset("x", "bang", "one") }
          .to raise_error("ERR value is not an integer or out of range")
      end
    end

    context "when the index is out of range" do
      it "returns an error" do
        redis.rpush("x", %w[1])

        expect { redis.lset("x", 1, "one") }
          .to raise_error("ERR index out of range")
        expect { redis.lset("x", -2, "one") }
          .to raise_error("ERR index out of range")
      end
    end
  end

  describe "LTRIM" do
    context "when the key does not exist" do
      it "returns OK" do
        expect(redis.ltrim("x", 0, -1)).to eql("OK")
      end
    end

    context "when the key exists" do
      def trim_list(min, max)
        results = redis.pipelined do |p|
          p.del("mylist")
          p.rpush("mylist", %w[1 2 3])
          p.ltrim("mylist", min, max)
          p.lrange("mylist", 0, -1)
        end

        expect(results[2]).to eql("OK")

        results.last
      end

      specify "ranging over the start index" do
        expect(trim_list(-4, -1)).to eql(%w[1 2 3])
        expect(trim_list(-3, -1)).to eql(%w[1 2 3])
        expect(trim_list(-2, -1)).to eql(%w[2 3])
        expect(trim_list(-1, -1)).to eql(%w[3])
        expect(trim_list(0, -1)).to eql(%w[1 2 3])
        expect(trim_list(1, -1)).to eql(%w[2 3])
        expect(trim_list(2, -1)).to eql(%w[3])
        expect(trim_list(3, -1)).to eql(%w[])
      end

      specify "ranging over the last index" do
        expect(trim_list(0, -4)).to eql(%w[])
        expect(trim_list(0, -3)).to eql(%w[1])
        expect(trim_list(0, -2)).to eql(%w[1 2])
        expect(trim_list(0, -1)).to eql(%w[1 2 3])
        expect(trim_list(0, 0)).to eql(%w[1])
        expect(trim_list(0, 1)).to eql(%w[1 2])
        expect(trim_list(0, 2)).to eql(%w[1 2 3])
        expect(trim_list(0, 3)).to eql(%w[1 2 3])
      end

      specify "ranging over both in lock step" do
        expect(trim_list(-5, -4)).to eql(%w[])
        expect(trim_list(-4, -3)).to eql(%w[1])
        expect(trim_list(-3, -2)).to eql(%w[1 2])
        expect(trim_list(-2, -1)).to eql(%w[2 3])
        expect(trim_list(-1, 0)).to eql(%w[])
        expect(trim_list(0, 1)).to eql(%w[1 2])
        expect(trim_list(1, 2)).to eql(%w[2 3])
        expect(trim_list(2, 3)).to eql(%w[3])
        expect(trim_list(3, 4)).to eql(%w[])
      end

      specify "when start > end it removes the key" do
        expect(trim_list(0, -4)).to eql(%w[])
        expect(redis.exists("mylist")).to be(false)
        expect(trim_list(4, 3)).to eql(%w[])
        expect(redis.exists("mylist")).to be(false)
        expect(trim_list(2, 1)).to eql(%w[])
        expect(redis.exists("mylist")).to be(false)
      end

      specify "when both are > len it removes the key" do
        expect(trim_list(3, 3)).to eql(%w[])
        expect(redis.exists("mylist")).to be(false)
      end

      specify "when both are < 0 it removes the key" do
        expect(trim_list(-4, -4)).to eql(%w[])
        expect(redis.exists("mylist")).to be(false)
      end
    end
  end

  describe "LREM" do
    context "when the key does not exist" do
      it "returns zero" do
        expect(redis.lrem("x", 0, "blah")).to eql(0)
      end
    end

    context "when the key exists" do
      specify "removing all occurrences" do
        redis.rpush("mylist", %w[foo bar foobar foobared zap bar test foo])
        expect(redis.lrem("mylist", 0, "bar")).to eql(2)
        expect(redis.lrange("mylist", 0, -1)).to eql(%w[foo foobar foobared zap test foo])
      end

      specify "removing the first occurrence" do
        redis.rpush("mylist", %w[foo bar foobar foobared zap bar test foo])
        expect(redis.lrem("mylist", 1, "bar")).to eql(1)
        expect(redis.lrange("mylist", 0, -1)).to eql(%w[foo foobar foobared zap bar test foo])
      end

      specify "trying to remove a non-existent element" do
        redis.rpush("mylist", %w[foo bar foobar foobared zap bar test foo])
        expect(redis.lrem("mylist", 1, "not-in-the-list")).to eql(0)
        expect(redis.lrange("mylist", 0, -1)).to eql(%w[foo bar foobar foobared zap bar test foo])
      end

      specify "removing from the back" do
        redis.rpush("mylist", %w[foo bar foobar foobared zap bar test foo])
        expect(redis.lrem("mylist", -1, "bar")).to eql(1)
        expect(redis.lrange("mylist", 0, -1)).to eql(%w[foo bar foobar foobared zap test foo])
        expect(redis.lrem("mylist", -2, "foo")).to eql(2)
        expect(redis.lrange("mylist", 0, -1)).to eql(%w[bar foobar foobared zap test])
      end

      specify "removing all values" do
        redis.rpush("mylist", %w[foo])
        expect(redis.lrem("mylist", 1, "foo")).to eql(1)
        expect(redis.exists("mylist")).to be(false)
      end
    end
  end
end
