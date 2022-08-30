#include <dmlc/strtonum.h>
#include <dmlc/logging.h>
#include <cstdlib>

int main(int argc, char *argv[]) {
  using namespace dmlc;

  // float
  std::vector<std::string> f = {
    "1234567901234", "+12345.6789", "-0.00123", "+0123.234e-2",
    "-234234.123123e20", "3.1029831e+38", "000.123e-28",
    "17.065995780200002000000", "0.00017065995780200002"};
  for (size_t i = 0; i < f.size(); ++i) {
    float v1 = dmlc::atof(f[i].c_str());
    float v2 = std::atof(f[i].c_str());
    CHECK_EQ(v1, v2);
  }

  // long
  std::vector<std::string> l = {
    "2147483647", "+12345", "-123123", "-2147483648"
  };
  for (size_t i = 0; i < l.size(); ++i) {
    long v1 = dmlc::atol(l[i].c_str());
    long v2 = std::atol(l[i].c_str());
    CHECK_EQ(v1, v2);
  }

  // uint64
  std::vector<std::string> ull = {
    "2147483647", "+12345", "18446744073709551615"
  };
  for (size_t i = 0; i < ull.size(); ++i) {
    unsigned long long v1 = dmlc::strtoull(ull[i].c_str(), 0, 10);
    unsigned long long v2 = std::strtoull(ull[i].c_str(), 0, 10);
    CHECK_EQ(v1, v2);
  }
  return 0;
}
