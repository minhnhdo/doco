/**
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package examples;

/**
 * Utility methods for working with log-likelihood
 */
public final class LogLikelihood {

  // private LogLikelihood() {
  // }
  public static double entropy(long... elements) {
    long sum = 0;
    double result = 0.0;
    for (long element : elements) {
      // Preconditions.checkArgument(element >= 0);
      result += xLogX(element);
      sum += element;
    }
    return xLogX(sum) - result;
  }

  private static double xLogX(long x) {
    return x == 0 ? 0.0 : x * Math.log(x);
  }

  public static double logLikelihoodRatio(long k11, long k12, long k21, long k22) {
    // Preconditions.checkArgument(k11 >= 0 && k12 >= 0 && k21 >= 0 && k22 >= 0);
    // note that we have counts here, not probabilities, and that the entropy is not normalized.
    double rowEntropy = entropy(k11 + k12, k21 + k22);
    double columnEntropy = entropy(k11 + k21, k12 + k22);
    double matrixEntropy = entropy(k11, k12, k21, k22);
    if (rowEntropy + columnEntropy < matrixEntropy) {
      // round off error
      return 0.0;
    }
    return 2.0 * (rowEntropy + columnEntropy - matrixEntropy);
  }
}
