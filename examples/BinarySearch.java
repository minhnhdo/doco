package examples;

public final class BinarySearch {
  // private BinarySearch() {}
  public static int binarySearchFromTo(byte[] array, byte value, int from, int to) {
    int mid = -1;
    while (from <= to) {
      mid = (from + to) >>> 1;
      if (value > array[mid]) {
        from = mid + 1;
      } else if (value == array[mid]) {
        return mid;
      } else {
        to = mid - 1;
      }
    }
    if (mid < 0) {
      return -1;
    }

    return -mid - (value < array[mid] ? 1 : 2);
  }
}
