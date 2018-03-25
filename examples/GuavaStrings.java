package examples;

public class GuavaStrings {
  public static <T> T checkNotNull(T reference) {
    if (reference == null) {
      throw new NullPointerException();
    }
    return reference;
  }

  static String format(String template, Object... args) {
    template = String.valueOf(template); // null -> "null"

    args = args == null ? new Object[] {"(Object[])null"} : args;

    // start substituting the arguments into the '%s' placeholders
    StringBuilder builder = new StringBuilder(template.length() + 16 * args.length);
    int templateStart = 0;
    int i = 0;
    while (i < args.length) {
      int placeholderStart = template.indexOf("%s", templateStart);
      if (placeholderStart == -1) {
        break;
      }
      builder.append(template, templateStart, placeholderStart);
      builder.append(args[i++]);
      templateStart = placeholderStart + 2;
    }
    builder.append(template, templateStart, template.length());

    // if we run out of placeholders, append the extra args in square braces
    if (i < args.length) {
      builder.append(" [");
      builder.append(args[i++]);
      while (i < args.length) {
        builder.append(", ");
        builder.append(args[i++]);
      }
      builder.append(']');
    }

    return builder.toString();
  }

  public static void checkArgument(
      boolean expression,
      String errorMessageTemplate,
      Object... errorMessageArgs) {
    if (!expression) {
      throw new IllegalArgumentException(format(errorMessageTemplate, errorMessageArgs));
    }
  }

  public static String repeat(String string, int count) {
    checkNotNull(string); // eager for GWT.

    if (count <= 1) {
      checkArgument(count >= 0, "invalid count: %s", count);
      return (count == 0) ? "" : string;
    }

    // IF YOU MODIFY THE CODE HERE, you must update StringsRepeatBenchmark
    final int len = string.length();
    final long longSize = (long) len * (long) count;
    final int size = (int) longSize;
    if (size != longSize) {
      throw new ArrayIndexOutOfBoundsException("Required array size too large: " + longSize);
    }

    final char[] array = new char[size];
    string.getChars(0, len, array, 0);
    int n;
    for (n = len; n < size - n; n <<= 1) {
      System.arraycopy(array, 0, array, n, n);
    }
    System.arraycopy(array, 0, array, n, size - n);
    return new String(array);
  }

	public void assertEquals(Object o1, Object o2) {
	}

	public void fail() {
	}

  public void testRepeat() {
    String input = "20";
    assertEquals("", GuavaStrings.repeat(input, 0));
    assertEquals("20", GuavaStrings.repeat(input, 1));
    assertEquals("2020", GuavaStrings.repeat(input, 2));
    assertEquals("202020", GuavaStrings.repeat(input, 3));

    assertEquals("", GuavaStrings.repeat("", 4));

    for (int i = 0; i < 100; ++i) {
      assertEquals(2 * i, GuavaStrings.repeat(input, i).length());
    }

    try {
      GuavaStrings.repeat("x", -1);
      fail();
    } catch (IllegalArgumentException expected) {
    }
    try {
      // Massive string
      GuavaStrings.repeat("12345678", (1 << 30) + 3);
      fail();
    } catch (ArrayIndexOutOfBoundsException expected) {
    }
  }

  // TODO: could remove if we got NPT working in GWT somehow
  public void testRepeat_null() {
    try {
      GuavaStrings.repeat(null, 5);
      fail();
    } catch (NullPointerException expected) {
    }
  }

  public static void main(String args[]) {
      GuavaStrings subject;

      subject = new GuavaStrings();
      subject.testRepeat();

      subject = new GuavaStrings();
      subject.testRepeat_null();
  }
}
