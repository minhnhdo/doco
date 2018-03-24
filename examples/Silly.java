package examples;

public class Silly {
  public static int silly(int a, int b) {
    if ((long)a < 0) {
      throw new RuntimeException("a is less than zero");
    }
    if ((long)b > 10) {
      throw new RuntimeException("b is more than 10");
    }
    if ((long)a < 2 && (long)b > 8) {
      throw new RuntimeException("a is less than two and b is more than 8");
    }
    return a + b;
  }
}
