#include <gtest/gtest.h>
#include <dmlc/parameter.h>
#include <vector>
#include <string>
#include <utility>
#include <cmath>

struct LearningParam : public dmlc::Parameter<LearningParam> {
  float float_param;
  double double_param;
  DMLC_DECLARE_PARAMETER(LearningParam) {
      DMLC_DECLARE_FIELD(float_param).set_default(0.01f);
      DMLC_DECLARE_FIELD(double_param).set_default(0.1);
  }
};

DMLC_REGISTER_PARAMETER(LearningParam);

TEST(Parameter, parsing_float) {
  LearningParam param;
  std::map<std::string, std::string> kwargs;

  kwargs["float_param"] = "0";
  param.Init(kwargs);
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "0.015625";  // can be represented exactly in IEEE 754
  ASSERT_NO_THROW(param.Init(kwargs));
  ASSERT_EQ(param.float_param, 0.015625f);
  kwargs["float_param"] = "-0.015625";  // can be represented exactly in IEEE 754
  ASSERT_NO_THROW(param.Init(kwargs));
  ASSERT_EQ(param.float_param, -0.015625f);

  kwargs["float_param"] = "1e-10";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "1e10";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "1.2f";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "1.2e-2f";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "3.4e+38";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "1.2e-38";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "16777216.01";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "4.920005e9";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["float_param"] = "4920000500.0";
  ASSERT_NO_THROW(param.Init(kwargs));

  // Range error should be caught
  kwargs["float_param"] = "1e-100";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "1e100";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "3.5e+38";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "1.1e-38";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);

  // Invalid inputs should be detected
  kwargs["float_param"] = "foobar";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "foo1.2";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "1.2e10foo";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "1.2e-2 foo";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);

  kwargs = std::map<std::string, std::string>();

  kwargs["double_param"] = "0";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "0.00048828125";  // can be represented exactly in IEEE 754
  ASSERT_NO_THROW(param.Init(kwargs));
  ASSERT_EQ(param.double_param, 0.00048828125);
  kwargs["double_param"] = "-0.00048828125";  // can be represented exactly in IEEE 754
  ASSERT_NO_THROW(param.Init(kwargs));
  ASSERT_EQ(param.double_param, -0.00048828125);

  kwargs["double_param"] = "1e-10";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1e10";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1.2f";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1.2e-2f";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1e-100";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1e100";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "1.7e+308";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "2.3e-308";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "16777217.01";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "100000000.01";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "9007199254740992.01";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "4.920005e9";
  ASSERT_NO_THROW(param.Init(kwargs));
  kwargs["double_param"] = "4920000500.0";
  ASSERT_NO_THROW(param.Init(kwargs));

  // Range error should be caught
  kwargs["double_param"] = "1e-500";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "1e500";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "1.8e+308";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "2.2e-308";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);

  // Invalid inputs should be detected
  kwargs["double_param"] = "foobar";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "foo1.2";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "1.2e10foo";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["double_param"] = "1.2e-2 foo";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);

  // INF and NAN
  kwargs = std::map<std::string, std::string>();
  errno = 0;  // clear errno, to clear previous range error
  for (const char* s : {
      "inf", "+inf", "-inf", "INF", "+INF", "-INF", "infinity", "+infinity",
      "-infinity", "INFINITY", "+INFINITY", "-INFINITY"}) {
    kwargs["float_param"] = s;
    ASSERT_NO_THROW(param.Init(kwargs));
    ASSERT_TRUE(std::isinf(param.float_param));
    kwargs["double_param"] = s;
    ASSERT_NO_THROW(param.Init(kwargs));
    ASSERT_TRUE(std::isinf(param.double_param));
  }
  for (const char* s : {
      "nan", "NAN", "nan(foobar)", "NAN(FooBar)", "NaN", "NaN(foo_bar_12)",
      "+nan", "+NAN", "+nan(foobar)", "+NAN(FooBar)", "+NaN", "+NaN(foo_bar_12)",
      "-nan", "-NAN", "-nan(foobar)", "-NAN(FooBar)", "-NaN",
      "-NaN(foo_bar_12)"}) {
    kwargs["float_param"] = s;
    ASSERT_NO_THROW(param.Init(kwargs));
    ASSERT_TRUE(std::isnan(param.float_param));
    kwargs["double_param"] = s;
    ASSERT_NO_THROW(param.Init(kwargs));
    ASSERT_TRUE(std::isnan(param.double_param));
  }
  kwargs["float_param"] = "infamous";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "infinity war";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
  kwargs["float_param"] = "Nanny";
  ASSERT_THROW(param.Init(kwargs), dmlc::ParamError);
}

TEST(Parameter, Update) {
  LearningParam param;
  using Args = std::vector<std::pair<std::string, std::string> >;
  auto unknown =
      param.UpdateAllowUnknown(Args{{"float_param", "0.02"},
                                    {"foo", "bar"}});
  ASSERT_EQ(unknown.size(), 1);
  ASSERT_EQ(unknown[0].first, "foo");
  ASSERT_EQ(unknown[0].second, "bar");
  ASSERT_NEAR(param.float_param, 0.02f, 1e-6);

  param.float_param = 0.02;
  param.UpdateAllowUnknown(Args{{"float_param", "0.02"},
                                {"foo", "bar"}});
  param.UpdateAllowUnknown(Args{{"foo", "bar"}});
  param.UpdateAllowUnknown(Args{{"double_param", "0.13"},
                                {"foo", "bar"}});
  ASSERT_NEAR(param.float_param, 0.02f, 1e-6);  // stays the same
  ASSERT_NEAR(param.double_param, 0.13, 1e-6);
}
