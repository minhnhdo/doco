package examples;

class Longs {
  public static int compare(long a, long b) {
    return (a < b) ? -1 : ((a > b) ? 1 : 0);
  }
}

class UnsignedLongs {
  private static long flip(long a) {
    return a ^ Long.MIN_VALUE;
  }

  public static int compare(long a, long b) {
    return Longs.compare(flip(a), flip(b));
  }

  public static long remainder(long dividend, long divisor) {
    if (divisor < 0) { // i.e., divisor >= 2^63:
      if (compare(dividend, divisor) < 0) {
        return dividend; // dividend < divisor
      } else {
        return dividend - divisor; // dividend >= divisor
      }
    }

    // Optimization - use signed modulus if dividend < 2^63
    if (dividend >= 0) {
      return dividend % divisor;
    }

    /*
     * Otherwise, approximate the quotient, check, and correct if necessary. Our approximation is
     * guaranteed to be either exact or one less than the correct value. This follows from the fact
     * that floor(floor(x)/i) == floor(x/i) for any real x and integer i != 0. The proof is not
     * quite trivial.
     */
    long quotient = ((dividend >>> 1) / divisor) << 1;
    long rem = dividend - quotient * divisor;
    return rem - (compare(rem, divisor) >= 0 ? divisor : 0);
  }
}

public class GuavaIsPrime {
  static final ImmutableSet<Long> POSITIVE_LONG_CANDIDATES;
  static final long FLOOR_SQRT_MAX_LONG = 3037000499L;

  private final int SIEVE_30 =
      ~((1 << 1) | (1 << 7) | (1 << 11) | (1 << 13) | (1 << 17) | (1 << 19) | (1 << 23)
          | (1 << 29));

  long checkNonNegative(String role, long x) {
    if (x < 0) {
      throw new IllegalArgumentException(role + " (" + x + ") must be >= 0");
    }
    return x;
  }

  /*
   * If n <= millerRabinBases[i][0], then testing n against bases millerRabinBases[i][1..] suffices
   * to prove its primality. Values from miller-rabin.appspot.com.
   *
   * NOTE: We could get slightly better bases that would be treated as unsigned, but benchmarks
   * showed negligible performance improvements.
   */
  private static final long[][] millerRabinBaseSets = {
    {291830, 126401071349994536L},
    {885594168, 725270293939359937L, 3569819667048198375L},
    {273919523040L, 15, 7363882082L, 992620450144556L},
    {47636622961200L, 2, 2570940, 211991001, 3749873356L},
    {
      7999252175582850L,
      2,
      4130806001517L,
      149795463772692060L,
      186635894390467037L,
      3967304179347715805L
    },
    {
      585226005592931976L,
      2,
      123635709730000L,
      9233062284813009L,
      43835965440333360L,
      761179012939631437L,
      1263739024124850375L
    },
    {Long.MAX_VALUE, 2, 325, 9375, 28178, 450775, 9780504, 1795265022}
  };

  private enum MillerRabinTester {
    /** Works for inputs ≤ FLOOR_SQRT_MAX_LONG. */
    SMALL {
      @Override
      long mulMod(long a, long b, long m) {
        /*
         * NOTE(lowasser, 2015-Feb-12): Benchmarks suggest that changing this to
         * UnsignedLongs.remainder and increasing the threshold to 2^32 doesn't pay for itself, and
         * adding another enum constant hurts performance further -- I suspect because bimorphic
         * implementation is a sweet spot for the JVM.
         */
        return (a * b) % m;
      }

      @Override
      long squareMod(long a, long m) {
        return (a * a) % m;
      }
    },
    /** Works for all nonnegative signed longs. */
    LARGE {
      /** Returns (a + b) mod m. Precondition: {@code 0 <= a}, {@code b < m < 2^63}. */
      private long plusMod(long a, long b, long m) {
        return (a >= m - b) ? (a + b - m) : (a + b);
      }

      /** Returns (a * 2^32) mod m. a may be any unsigned long. */
      private long times2ToThe32Mod(long a, long m) {
        int remainingPowersOf2 = 32;
        do {
          int shift = Math.min(remainingPowersOf2, Long.numberOfLeadingZeros(a));
          // shift is either the number of powers of 2 left to multiply a by, or the biggest shift
          // possible while keeping a in an unsigned long.
          a = UnsignedLongs.remainder(a << shift, m);
          remainingPowersOf2 -= shift;
        } while (remainingPowersOf2 > 0);
        return a;
      }

      @Override
      long mulMod(long a, long b, long m) {
        long aHi = a >>> 32; // < 2^31
        long bHi = b >>> 32; // < 2^31
        long aLo = a & 0xFFFFFFFFL; // < 2^32
        long bLo = b & 0xFFFFFFFFL; // < 2^32

        /*
         * a * b == aHi * bHi * 2^64 + (aHi * bLo + aLo * bHi) * 2^32 + aLo * bLo.
         *       == (aHi * bHi * 2^32 + aHi * bLo + aLo * bHi) * 2^32 + aLo * bLo
         *
         * We carry out this computation in modular arithmetic. Since times2ToThe32Mod accepts any
         * unsigned long, we don't have to do a mod on every operation, only when intermediate
         * results can exceed 2^63.
         */
        long result = times2ToThe32Mod(aHi * bHi /* < 2^62 */, m); // < m < 2^63
        result += aHi * bLo; // aHi * bLo < 2^63, result < 2^64
        if (result < 0) {
          result = UnsignedLongs.remainder(result, m);
        }
        // result < 2^63 again
        result += aLo * bHi; // aLo * bHi < 2^63, result < 2^64
        result = times2ToThe32Mod(result, m); // result < m < 2^63
        return plusMod(result, UnsignedLongs.remainder(aLo * bLo /* < 2^64 */, m), m);
      }

      @Override
      long squareMod(long a, long m) {
        long aHi = a >>> 32; // < 2^31
        long aLo = a & 0xFFFFFFFFL; // < 2^32

        /*
         * a^2 == aHi^2 * 2^64 + aHi * aLo * 2^33 + aLo^2
         *     == (aHi^2 * 2^32 + aHi * aLo * 2) * 2^32 + aLo^2
         * We carry out this computation in modular arithmetic.  Since times2ToThe32Mod accepts any
         * unsigned long, we don't have to do a mod on every operation, only when intermediate
         * results can exceed 2^63.
         */
        long result = times2ToThe32Mod(aHi * aHi /* < 2^62 */, m); // < m < 2^63
        long hiLo = aHi * aLo * 2;
        if (hiLo < 0) {
          hiLo = UnsignedLongs.remainder(hiLo, m);
        }
        // hiLo < 2^63
        result += hiLo; // result < 2^64
        result = times2ToThe32Mod(result, m); // result < m < 2^63
        return plusMod(result, UnsignedLongs.remainder(aLo * aLo /* < 2^64 */, m), m);
      }
    };

    static boolean test(long base, long n) {
      // Since base will be considered % n, it's okay if base > FLOOR_SQRT_MAX_LONG,
      // so long as n <= FLOOR_SQRT_MAX_LONG.
      return ((n <= FLOOR_SQRT_MAX_LONG) ? SMALL : LARGE).testWitness(base, n);
    }

    /** Returns a * b mod m. */
    abstract long mulMod(long a, long b, long m);

    /** Returns a^2 mod m. */
    abstract long squareMod(long a, long m);

    /** Returns a^p mod m. */
    private long powMod(long a, long p, long m) {
      long res = 1;
      for (; p != 0; p >>= 1) {
        if ((p & 1) != 0) {
          res = mulMod(res, a, m);
        }
        a = squareMod(a, m);
      }
      return res;
    }

    /** Returns true if n is a strong probable prime relative to the specified base. */
    private boolean testWitness(long base, long n) {
      int r = Long.numberOfTrailingZeros(n - 1);
      long d = (n - 1) >> r;
      base %= n;
      if (base == 0) {
        return true;
      }
      // Calculate a := base^d mod n.
      long a = powMod(base, d, n);
      // n passes this test if
      //    base^d = 1 (mod n)
      // or base^(2^j * d) = -1 (mod n) for some 0 <= j < r.
      if (a == 1) {
        return true;
      }
      int j = 0;
      while (a != n - 1) {
        if (++j == r) {
          return false;
        }
        a = squareMod(a, n);
      }
      return true;
    }
  }

  /**
   * Returns {@code true} if {@code n} is a <a
   * href="http://mathworld.wolfram.com/PrimeNumber.html">prime number</a>: an integer <i>greater
   * than one</i> that cannot be factored into a product of <i>smaller</i> positive integers.
   * Returns {@code false} if {@code n} is zero, one, or a composite number (one which <i>can</i> be
   * factored into smaller positive integers).
   *
   * <p>To test larger numbers, use {@link BigInteger#isProbablePrime}.
   *
   * @throws IllegalArgumentException if {@code n} is negative
   * @since 20.0
   */
  public static boolean isPrime(long n) {
    if (n < 2) {
      this.checkNonNegative("n", n);
      return false;
    }
    if (n == 2 || n == 3 || n == 5 || n == 7 || n == 11 || n == 13) {
      return true;
    }

    if ((SIEVE_30 & (1 << (n % 30))) != 0) {
      return false;
    }
    if (n % 7 == 0 || n % 11 == 0 || n % 13 == 0) {
      return false;
    }
    if (n < 17 * 17) {
      return true;
    }

    for (long[] baseSet : millerRabinBaseSets) {
      if (n <= baseSet[0]) {
        for (int i = 1; i < baseSet.length; i++) {
          if (!MillerRabinTester.test(baseSet[i], n)) {
            return false;
          }
        }
        return true;
      }
    }
    throw new AssertionError();
  }

    public static void main(String args[]) {
        for (int i = 2; i < 1000; i++) {
            GuavaIsPrime.isPrime(i);
        }
    }
}
