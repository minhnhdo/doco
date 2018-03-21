package examples;

public class IsPositive {
  protected int state;
  public int isPositive(IsPositive other, int i) {
    if (i <= other.state) {
      return 0;
    }
    return 1;
  }
  public int countPositives(int[] xs) {
    int cnt = 0;
    for (int i = 0; i < xs.length; i++) {
      cnt += isPositive(this, xs[i]);
    }
    if (cnt == 3) {
      throw new RuntimeException("Three positives!");
    }
    return cnt;
  }
}
